#version 460

struct SynthData
{
    uint t;
};

layout(local_size_x_id = 1, local_size_y_id = 2, local_size_z = 1) in;
layout(set = 0, binding = 0) buffer OutData {
    float data[];
} data;

layout(set = 0, binding = 1) buffer InData {
    SynthData synth_data[];
} synth_data;

const float PI = 3.1415926535897932384626433832795;
layout(constant_id = 0) const uint sample_rate = 48000;
layout(constant_id = 3) const uint num_channels = 2;



void main() {
    // Time in samples
    uint t = gl_GlobalInvocationID.x + synth_data.synth_data[0].t;

    // Current channel to write to
    uint channel = gl_GlobalInvocationID.y;
    
    uint size = gl_WorkGroupSize.x*gl_NumWorkGroups.x;
    // The actual expression
    data.data[gl_GlobalInvocationID.x + size*channel] = sin((440 + 220*channel)*2*PI*mod(t/float(sample_rate), 1));
}
