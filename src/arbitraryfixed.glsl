#extension GL_EXT_shader_explicit_arithmetic_types_int64: enable

#include "../target/constants.glsl"

/* !TODO! */
/* Consider avoiding the use of uint64_t in favour of a better rollover detection */
/* Trig functions will be a pain in the ass, CORDIC? */
/* sqrt log cos */

void fix_add(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    bool carry_prev = false;

    for (uint i = 0; i < SIZE; ++i) {
        r[i] = a[i] + b[i];
        
        // detect overflow
        bool carry = (r[i] < a[i]);
        // add previous overflow
        r[i] += uint(carry_prev);
        // detect overflow on carry
        carry_prev = carry || (carry_prev && r[i] == 0);
    }
}

void fix_increment(inout uint r[SIZE]) {
    bool carry_prev = true;

    for (uint i = 0; i < SIZE; ++i) {
        // add overflow
        r[i] += uint(carry_prev);
        // detect overflow on carry
        carry_prev = carry_prev && r[i] == 0;
    }
}

bool is_negative(in uint a[SIZE]) {
    return (a[SIZE - 1] & 0x80000000) > 0;
}

void fix_negate_in_place(inout uint r[SIZE]) {
    for (uint i = 0; i < SIZE; ++i) {
        r[i] = ~r[i];
    }

    fix_increment(r);
}

// !!TODO!! find a less shitty algorithm that isn't O(n^2)
void fix_mul(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    uint res[2*SIZE];

    for (int i = 0; i < 2*SIZE; ++i) {
        res[i] = 0;
    }

    bool a_is_negative = is_negative(a);
    bool b_is_negative = is_negative(b);

    if (a_is_negative) {
        fix_negate_in_place(a);
    }

    if (b_is_negative) {
        fix_negate_in_place(b);
    }

    for (uint i = 0; i < SIZE; ++i) {
        uint carry = 0;
        for (uint j = 0; j < SIZE; ++j) {
            uint64_t product = uint64_t(a[i]) * uint64_t(b[j]) + uint64_t(res[i + j]) + uint64_t(carry);
            res[i + j] = uint(product);
            carry = uint(product >> 32);
        }
        res[i + SIZE] = carry;
    }


    // Shift right by SIZE across word boundaries
    for (int i = int(SIZE) - 1; i >= 0; --i) {
        r[i] = (((SCALING_FACTOR & 0x1F) > 0)?(res[i + 1 + (SCALING_FACTOR/32)] << ((-SCALING_FACTOR) & 0x1F)):0) |
               (res[i + (SCALING_FACTOR/32)] >> ((SCALING_FACTOR & 0x1F)));
    }

    // A NEGATIVE TIMES A NEGATIVE IS A POSITIVE,
    // AGAIN,
    // A NEGATIVE TIMES A NEGATIVE IS A POSITIVE
    if (a_is_negative != b_is_negative) {
        fix_negate_in_place(r);
    }
}

void fix_sub(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    fix_negate_in_place(b);
    fix_add(r, a, b);
}

float fix_to_float(in uint a[SIZE]) {
    float res = 0;

    bool a_negative = is_negative(a);
    if (a_negative) {
        fix_negate_in_place(a);
    }

    for (int i = 0; i < SIZE; ++i) {
        float factor = exp2(32*i - int(SCALING_FACTOR));

        if (!isinf(factor)) {
            res += a[i] * factor;
        }
    }

    return a_negative?-res:res;
}

void fix_copy(out uint r[SIZE], in uint a[SIZE]) {
    for (int i = 0; i < SIZE; ++i) {
        r[i] = a[i];
    }
}

void fix_from_float(out uint r[SIZE], in float a) {
    uint a_int = floatBitsToUint(a);
    bool sign = bool(a_int & 0x80000000);
    int exponent = int((a_int & 0x7f800000) >> 23);
    uint mantissa_complete = (a_int & 0x007fffff) + (1 << 23);

    for (int i = 0; i < SIZE; ++i) {
        r[i] = 0;
    }

    int offset = int(SCALING_FACTOR) + (exponent - 127 - 23);
    if (offset >= 0) {
        r[uint(offset)/32] = mantissa_complete << (offset & 0x1F);
    }
    if (((SCALING_FACTOR & 0x1F) != 0) && (offset >= -1)) {
        r[uint(offset)/32 + 1] = mantissa_complete >> ((-int(offset)) & 0x1F);
    }

    // Two's compliment representation, flip the bits and add one
    if (sign) {
        fix_negate_in_place(r);
    }
}

void fix_from_uint(out uint r[SIZE], in uint a) {
    for (int i = 0; i < SIZE; ++i) {
        r[i] = 0;
    }
    r[SCALING_FACTOR/32] = a << (SCALING_FACTOR & 0x1F);
    if ((SCALING_FACTOR & 0x1F) != 0) {
        r[SCALING_FACTOR/32 + 1] = a >> ((-int(SCALING_FACTOR)) & 0x1F);
    }
}

uint fix_divide_by_u32(out uint r[SIZE], in uint a[SIZE], in uint b) {
    bool a_is_negative = is_negative(a);

    if (a_is_negative) {
        fix_negate_in_place(a);
    }

    //  Make division go brr
    uint64_t temp = 0;
    for (int i = int(SIZE) - 1; i >= 0; --i) {
        temp <<= 32;
        temp |= a[i];
        r[i] = uint(temp / b);
        temp -= r[i] * b;
    }

    if (a_is_negative) {
        fix_negate_in_place(r);
    }

    return uint(temp);
}

void fix_rshift1_double(out uint r[2*SIZE], in uint a[2*SIZE]) {
    r[2*SIZE - 1] = a[2*SIZE - 1] >> 1;
    for (int i = 2*int(SIZE) - 2; i >= 0; --i) {
        r[i] = (a[i + 1] << 31) | (a[i] >> 1);
    }
}

void fix_lshift1_double(out uint r[2*SIZE], in uint a[2*SIZE]) {
    r[0] = a[0] << 1;
    for (int i = 1; i < 2*SIZE; ++i) {
        r[i] = (a[i] << 1) | (a[i - 1] >> 31);
    }
}

void fix_rshift1(out uint r[SIZE], in uint a[SIZE]) {
    r[SIZE - 1] = a[SIZE - 1] >> 1;
    for (int i = int(SIZE) - 2; i >= 0; --i) {
        r[i] = (a[i + 1] << 31) | (a[i] >> 1);
    }
}

void fix_lshift1(out uint r[SIZE], in uint a[SIZE]) {
    r[0] = a[0] << 1;
    for (int i = 1; i < SIZE; ++i) {
        r[i] = (a[i] << 1) | (a[i - 1] >> 31);
    }
}

int fix_compare(in uint a[SIZE], in uint b[SIZE]) {
    fix_sub(a, a, b);

    bool is_zero = true;
    for (int i = 0; i < SIZE; ++i) {
        is_zero = is_zero && (a[i] > 0);
    }

    if (is_zero) { 
        return 0;
    } else if (is_negative(a)) {
        return -1;
    } else {
        return 1;
    }
}

void fix_sub_double(out uint r[2*SIZE], in uint a[2*SIZE], in uint b[2*SIZE]) {
    bool carry_prev = true;
    for (uint i = 0; i < 2*SIZE; ++i) {
        b[i] = ~b[i];
        // add overflow
        b[i] += uint(carry_prev);
        // detect overflow on carry
        carry_prev = carry_prev && b[i] == 0;
    }

    carry_prev = false;

    for (uint i = 0; i < 2*SIZE; ++i) {
        r[i] = a[i] + b[i];
        
        // detect overflow
        bool carry = (r[i] < a[i]);
        // add previous overflow
        r[i] += uint(carry_prev);
        // detect overflow on carry
        carry = carry || (carry_prev && r[i] == 0);

        carry_prev = carry;
    }
}


int fix_compare_double(in uint a[2*SIZE], in uint b[2*SIZE]) {
    fix_sub_double(a, a, b);

    bool is_zero = true;
    for (int i = 0; i < 2*SIZE; ++i) {
        is_zero = is_zero && (a[i] > 0);
    }

    if (is_zero) { 
        return 0;
    } else if ((a[2*SIZE - 1] & 0x80000000) > 0) {
        return -1;
    } else {
        return 1;
    }
}

void fix_div(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    bool a_is_negative = is_negative(a);
    bool b_is_negative = is_negative(b);

    if (a_is_negative) {
        fix_negate_in_place(a);
    }

    if (b_is_negative) {
        fix_negate_in_place(b);
    }

    uint rem[2*SIZE];
    uint D[2*SIZE];
    uint res[2*SIZE];
    uint rem_d[2*SIZE];
    for (int i = 0; i < SIZE; ++i) {
        rem[i] = a[i];
        rem[i + SIZE] = 0;
        D[i + SIZE] = b[i];
        D[i] = 0;
        res[i] = 0;
        res[i + SIZE] = 0;
    }

    // Horrible algorithm
    for (int i = 2 * int(SIZE) * 32; i >= 0; --i) {
        fix_lshift1_double(res, res);
        fix_sub_double(rem_d, rem, D);
        if ((rem_d[2 * SIZE - 1] & 0x80000000) == 0) {
            res[0] |= 1;
            rem = rem_d;
        }

        fix_rshift1_double(D, D);
    }

    // Shift back into place
    for (int i = 0; i < SIZE; ++i) {
        r[i] = (((SCALING_FACTOR & 0x1F) > 0)?(res[i + SIZE - 1 - (SCALING_FACTOR/32)] >> ((-SCALING_FACTOR) & 0x1F)):0) |
               (res[i + SIZE - (SCALING_FACTOR/32)] << ((SCALING_FACTOR & 0x1F)));
    }

    if (a_is_negative != b_is_negative) {
        fix_negate_in_place(r);
    }
}

// find most significant bit, get closest two bit aligned value and set value bitshifted right by x/2 as initial guess
// Only takes +ve numbers
void fix_sqrt(out uint r[SIZE], in uint a[SIZE]) {
    uint tmp[SIZE];
    // Zero out result
    for (int i = 0; i < SIZE; ++i) {
        r[i] = 0;
        tmp[i] = 0;
    }

    // Ensure no divide by zeros
    bool is_zero = true;
    for (int i = 0; i < SIZE; ++i) {
        is_zero = is_zero && (a[i] == 0);
    }

    if (is_zero) {
        return;
    }

    // Find most significant bit divided by two
    uint msb_div2 = SIZE * 16;
    for (int i = int(SIZE) - 1; i >= 0; ++i) {
        if (a[i] > 0) {
            uint k = a[i];
            while (k != 0) {
                k >>= 2;
                ++msb_div2;
            }
            break;
        }

        msb_div2 -= 16;
    }

    // Set the guess
    r[msb_div2/32] = 1 << (msb_div2 & 0x1F);

    // TODO: tune to SIZE, should be 5 + log_2(SIZE)
    const uint iterations = 8;
    // Do Newton's method
    for (uint k = 0; k < iterations; ++k) {
        fix_mul(tmp, r, r); // x^2
        fix_add(tmp, tmp, a); // + a
        fix_div(tmp, tmp, r); // /x
        fix_rshift1(r, tmp); // /2
    }
}

void fix_truncate(inout uint r[SIZE], in uint b) {
    for (int i = int(SIZE) - 1; i >= int(SIZE) - (b/32); --i) {
        r[i] = 0;
    }

    r[SIZE - 1 - b/32] &= 0xFFFFFFFF >> (b & 0x1F);
}

void fix_rem(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    fix_div(r, a, b);

    r[SCALING_FACTOR / 32] &= 0xFFFFFFFF << (SCALING_FACTOR & 0x1F);
    for (int i = 0; i < SCALING_FACTOR / 32; ++i) {
        r[i] = 0;
    }

    fix_mul(r, r, b);
    fix_sub(r, a, r);
}

// TODO, need a pi constant not precise enough
// TODO, also 1 const
void fix_cos(out uint r[SIZE], in uint a[SIZE]) { 
    fix_lshift1(r, FIX_PI);
    fix_rem(r, a, r);
    fix_sub(r, r, FIX_PI);
    bool is_neg = is_negative(r);
    fix_rem(a, a, FIX_PI);
    fix_mul(a, a, a);

    fix_copy(r, taylor_table[0]);

    for (int i = 1; i < TRIG_PRECISION; ++i) {
        fix_mul(r, r, a);
        fix_add(r, r, taylor_table[i]);
    }

    fix_mul(r, r, a);

    uint ONE[SIZE];
    fix_from_uint(ONE, 1);

    fix_add(r, r, ONE);

    if (!is_neg) {
        fix_negate_in_place(r);
    }
}

// shift a until in range of 1 to 2
// Taylor series go brrrrrr
void fix_ln(out uint r[SIZE], in uint a[SIZE]) {
    // TODO
}
