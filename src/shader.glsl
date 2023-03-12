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
layout(constant_id = 0) const float sample_rate = 48000.0;
layout(constant_id = 3) const uint num_channels = 2;

void main() {
    uint t = gl_GlobalInvocationID.x + synth_data.synth_data[0].t;
    uint c = gl_GlobalInvocationID.y;
    data.data[(gl_GlobalInvocationID.x * num_channels) + c] = sin(440 + 220*c * 2 * PI * mod(t, sample_rate)/sample_rate);
}
