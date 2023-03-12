use std::{mem::MaybeUninit, sync::Arc};

use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

use bytemuck::{Pod, Zeroable};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Consumer, HeapRb, SharedRb};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct SynthData {
    t: u32,
}

const DATA_BUFFER_SAMPLES: u32 = 8192;
const SAMPLE_RATE: u32 = 48000;
const CHANNELS: u32 = 2;
const DATA_BUFFER_SIZE: u32 = DATA_BUFFER_SAMPLES * CHANNELS;

fn create_device() -> (Arc<Device>, Arc<Queue>) {
    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            // Enable enumerating devices that use non-conformant vulkan implementations. (ex. MoltenVK)
            enumerate_portability: true,
            ..Default::default()
        },
    )
    .unwrap();

    // Choose which physical device to use.
    let device_extensions = DeviceExtensions {
        khr_storage_buffer_storage_class: true,
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
            // that supports compute operations.
            p.queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.compute)
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type
    );

    // Now initializing the device.
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    (device, queues.next().unwrap())
}

fn get_subgroup_size(device: Arc<Device>) -> u32 {
    match device.physical_device().properties().subgroup_size {
        Some(subgroup_size) => {
            println!("Subgroup size is {subgroup_size}");
            subgroup_size
        }
        None => {
            println!("This Vulkan driver doesn't provide physical device Subgroup information");
            64
        }
    }
}

fn create_command_buffers(
    device: Arc<Device>,
    queue: Arc<Queue>,
    data_buffers: Vec<Arc<CpuAccessibleBuffer<[f32]>>>,
    parameter_buffer: Arc<CpuAccessibleBuffer<[SynthData]>>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    let subgroup_size = get_subgroup_size(device.clone());
    let pipeline = {
        mod cs {
            vulkano_shaders::shader! {
                ty: "compute",
                path: "src/shader.glsl"
            }
        }

        let shader = cs::load(device.clone()).unwrap();

        let spec_consts = cs::SpecializationConstants {
            constant_1: subgroup_size, // local_size_x
            constant_2: CHANNELS,      // local_size_y
            sample_rate: SAMPLE_RATE as f32,
            num_channels: CHANNELS
        };
        ComputePipeline::new(
            device.clone(),
            shader.entry_point("main").unwrap(),
            &spec_consts,
            None,
            |_| {},
        )
        .unwrap()
    };
    let layout = pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let sets = data_buffers.iter().map(|buf| {
        PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, buf.clone()),
                WriteDescriptorSet::buffer(1, parameter_buffer.clone()),
            ],
        )
        .unwrap()
    });
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());
    // In order to execute our operation, we have to build a command buffer.
    let builders = sets.map(|set| {
        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();
        builder
            .bind_pipeline_compute(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([DATA_BUFFER_SAMPLES / subgroup_size, 1, 1])
            .unwrap();
        builder
    });

    builders
        .map(|builder| Arc::new(builder.build().unwrap()))
        .collect::<Vec<_>>()
}

fn create_output_stream(
    mut consumer: Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
) -> cpal::Stream {
    let cpal_device = cpal::default_host()
        .default_output_device()
        .expect("failed to find output device");
    println!("Output device: {}", cpal_device.name().unwrap());

    let stream_config = cpal::StreamConfig {
        channels: CHANNELS as u16,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Fixed(1024),
    };
    println!("Default output config: {:?}", stream_config);

    let output_stream = cpal_device
        .build_output_stream(
            &stream_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                consumer.pop_slice(data);
            },
            |err: cpal::StreamError| {
                eprintln!("an error occurred on stream: {}", err);
            },
            None,
        )
        .unwrap();

    output_stream.play().unwrap();

    output_stream
}

fn main() {
    let (device, queue) = create_device();

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    // We start by creating the buffer that will store the data.
    let data_buffers = (0..2)
        .map(|_| {
            // Iterator that produces the data.
            let data_iter = (0..DATA_BUFFER_SIZE).map(|_| 0f32);

            // Builds the buffer and fills it with this iterator.
            CpuAccessibleBuffer::from_iter(
                &memory_allocator,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                true,
                data_iter,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let parameter_buffer = {
        let data_iter = (0..1).map(|_| SynthData { t: 0 });

        CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                storage_buffer: true,
                ..BufferUsage::empty()
            },
            true,
            data_iter,
        )
        .unwrap()
    };

    let command_buffers = create_command_buffers(
        device.clone(),
        queue.clone(),
        data_buffers.clone(),
        parameter_buffer.clone(),
    );

    let ring = HeapRb::new(DATA_BUFFER_SIZE as usize * 2);
    let (mut producer, consumer) = ring.split();

    let data_length = DATA_BUFFER_SAMPLES * 64;

    let _output_stream = create_output_stream(consumer);

    for (prev_data, next_command) in data_buffers
        .into_iter()
        .cycle()
        .zip(command_buffers.into_iter().cycle().skip(1))
        .take((data_length / DATA_BUFFER_SAMPLES) as usize)
    {
        let future = sync::now(device.clone())
            .then_execute(queue.clone(), next_command)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        // Wait for room in the buffer
        while producer.free_len() < DATA_BUFFER_SIZE as usize {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        producer.push_slice(&prev_data.read().unwrap());

        // Update parameters
        future.wait(None).unwrap();
        {
            let mut synth_parameter_content = parameter_buffer.write().unwrap();
            synth_parameter_content[0].t += DATA_BUFFER_SAMPLES;
        }
    }

    // Wait for the stream to clear before dropping the output stream
    while producer.len() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("Success!");
}
