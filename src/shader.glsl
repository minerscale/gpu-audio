#version 460

//#include "arbitraryfixed.glsl"

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

float theta(float t) {
    return (t/2.0)*(log(t/(2.0*PI)) - 1.0) - PI/8.0 + 1.0/(48.0*t) + 7.0/(5760.0*t*t*t);
}

int alternate(int a) {
    return 2*a - 4*(a/2) - 1;
}

int v(float t) {
    return int(sqrt(t/(2.0*PI)));
}

float Psi(float t) {
    return cos(2.0*PI*(t*t - t - 1.0/16.0))/cos(2.0*PI*t);
}

float c_0(float t, int v) {
    return Psi(sqrt(t/(2.0*PI)) - v);
}

float R(float t, int v) {
    return alternate(v) * pow((2.0*PI/t), 1.0/4.0) * c_0(t, v);
}

float Z(float t, int v) {
    float result = 0.0;

    float theta_t = theta(t);
    // TODO: branching bullshit makes this a bad idea, find a way to make this condition parallel.
    for (int k = 1; k <= v; ++k) {
        result += inversesqrt(float(k)) * cos(theta_t - t*log(k));
    }

    return 2.0*result + R(t, v);
}

vec2 zeta(float t) {
    int v = v(t);
    float Z = Z(t, v);
    float theta = -theta(t);
    float volume = 0.1;
    return vec2(volume * (Z * cos(theta)), volume * (Z * sin(theta)));
}

void main() {
    // Time in samples
    uint t = gl_GlobalInvocationID.x + synth_data.synth_data[0].t;

    // Current channel to write to
    uint channel = gl_GlobalInvocationID.y;
    
    uint size = gl_WorkGroupSize.x*gl_NumWorkGroups.x;
    // The actual expression
    //float t_norm = 800.0 * (float(t)/float(sample_rate));
    
    data.data[gl_GlobalInvocationID.x + size*channel] = sin((440 + 220*channel)*2*PI*mod(t/float(sample_rate), 1));

    //vec2 zeta_1 = zeta(t_norm);
    //vec2 zeta_2 = zeta((3.0/2.0) * (t_norm + 4.0*800.0));

    //data.data[gl_GlobalInvocationID.x] = zeta_1.x + zeta_2.x;
    //if (num_channels > 1) {
    //    data.data[gl_GlobalInvocationID.x + size] = zeta_1.y + zeta_2.y;
    //}
}
