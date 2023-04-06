mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shader.glsl",
        include: [INCLUDE_PATH]
    }
}
