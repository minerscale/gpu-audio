#version 460

const float PI = 3.1415926535897932384626433832795;
#include "arbitrary_fixed_glsl/arbitraryfixed.glsl"

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


layout(constant_id = 0) const uint sample_rate = 48000;
layout(constant_id = 3) const uint num_channels = 2;

uint v(in uint t[SIZE]) {
    fix_div(t, t, FIX_2_PI);
    fix_sqrt(t, t);
    fix_floor(t);
    return fix_to_uint(t);
}

void Psi(out uint r[SIZE], in uint t[SIZE]) {
    uint tmp[SIZE];

    fix_mul(r, t, t);
    fix_sub(r, r, t);
    fix_from_float(tmp, 1.0/16.0);
    fix_sub(r, r, tmp);
    fix_mul(r, r, FIX_2_PI);
    fix_sub(r, FIX_PI_2, r);
    fix_sin(r, r);
    fix_mul(tmp, FIX_2_PI, t);
    fix_sub(tmp, FIX_PI_2, tmp);
    fix_sin(tmp, tmp);
    fix_div(r, r, tmp);
}

void theta(out uint r[SIZE], in uint t[SIZE]) {
    uint tmp_1[SIZE];

    fix_div(r, t, FIX_2_PI);
    fix_ln(r, r);
    
    fix_add(r, r, FIX_NEG_ONE);
    fix_mul(r, r, t);

    fix_rshift1(tmp_1, FIX_PI_2);
    fix_sub(r, r, tmp_1);
    fix_rshift1(r, r);
}

void R(out uint r[SIZE], in uint fix_t[SIZE], in int v) {
    uint tmp_1[SIZE];

    fix_div(tmp_1, FIX_2_PI, fix_t);
    fix_sqrt(tmp_1, tmp_1);
    fix_sqrt(tmp_1, tmp_1);

    fix_div(r, fix_t, FIX_2_PI);
    fix_sqrt(r, r);
    fix_from_uint(fix_t, v);
    fix_sub(r, r, fix_t);
    Psi(r, r);

    fix_mul(r, tmp_1, r);

    fix_cond_negate(r, (v % 2) == 0);
}

void Z(out uint r[SIZE], in uint fix_t[SIZE], in uint fix_theta[SIZE], in int v) {
    uint tmp_1[SIZE];
    uint tmp_3[SIZE];
    uint fix_k[SIZE];
    
    r = FIX_ZERO;
    fix_k = FIX_ZERO;
    for (uint k = 1; k <= INV_SQRT_SIZE; ++k) {
        fix_add(fix_k, fix_k, FIX_ONE);
        
        fix_ln(tmp_3, fix_k);
        fix_mul(tmp_3, tmp_3, fix_t);
        fix_sub(tmp_3, fix_theta, tmp_3);
        fix_sub(tmp_3, FIX_PI_2, tmp_3);
        fix_sin(tmp_3, tmp_3);
        
        fix_mul(tmp_1, tmp_3, inv_sqrt_table[k - 1]);
        
        fix_cond_wipe(tmp_1, v >= k);
        fix_add(r, r, tmp_1);
    }

    fix_lshift1(r, r);
}

vec2 zeta(in uint fix_t[SIZE]) {
    float t = fix_to_float(fix_t);
    int v = int(v(fix_t));
    uint fix_theta[SIZE];
    theta(fix_theta, fix_t);

    uint z[SIZE];
    uint tmp[SIZE];
    Z(z, fix_t, fix_theta, v);

    R(tmp, fix_t, v);
    fix_add(z, z, tmp);

    fix_neg(fix_theta);

    fix_sub(tmp, FIX_PI_2, fix_theta);
    fix_sin(tmp, tmp);
    fix_mul(tmp, z, tmp);
    float x = fix_to_float(tmp);

    fix_sin(tmp, fix_theta);
    fix_mul(tmp, z, tmp);
    float y = fix_to_float(tmp);

    return vec2(x, y);
}

void main() {
    // Time in samples
    uint t = gl_GlobalInvocationID.x + synth_data.synth_data[0].t;

    // Current channel to write to
    uint channel = gl_GlobalInvocationID.y;
    
    uint size = gl_WorkGroupSize.x*gl_NumWorkGroups.x;
    // The actual expression
    float t_norm = (float(800.0 * t)/float(sample_rate));

    uint samp[SIZE];
    uint tmp[SIZE];
    fix_from_uint(tmp, 800);
    fix_from_uint(samp, t);
    fix_mul(samp, samp, tmp);
    fix_from_uint(tmp, sample_rate);
    fix_div(samp, samp, tmp);
    //fix_from_float(samp, t_norm);
    //fix_floor(samp);
    //float a = 0.1 * fix_to_float(samp);
    //data.data[gl_GlobalInvocationID.x] = a;
    //data.data[gl_GlobalInvocationID.x + size] = sin(t_norm);

    vec2 zeta = 0.025 * zeta(samp);

    data.data[gl_GlobalInvocationID.x] = zeta.x;
    if (num_channels > 1) {
        data.data[gl_GlobalInvocationID.x + size] = zeta.y;
    }
}
