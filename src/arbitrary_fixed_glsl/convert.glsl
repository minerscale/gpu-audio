float fix_to_float(in uint a[SIZE]) {
    float res = 0;

    bool a_negative = fix_is_negative(a);
    if (a_negative) {
        fix_neg(a);
    }

    for (int i = 0; i < SIZE; ++i) {
        float factor = exp2(32*i - int(SCALING_FACTOR));

        if (!isinf(factor)) {
            res += a[i] * factor;
        }
    }

    return a_negative?-res:res;
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
    if (((offset & 0x1F) != 0) && (offset >= -1)) {
        r[uint(offset)/32 + 1] = mantissa_complete >> ((-int(offset)) & 0x1F);
    }

    // Two's compliment representation, flip the bits and add one
    if (sign) {
        fix_neg(r);
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

void fix_from_int(out uint r[SIZE], in int a) {
    bool a_negative = a < 0;
    if (a_negative) {
        a = -a;
    }
    for (int i = 0; i < SIZE; ++i) {
        r[i] = 0;
    }
    r[SCALING_FACTOR/32] = a << (SCALING_FACTOR & 0x1F);
    if ((SCALING_FACTOR & 0x1F) != 0) {
        r[SCALING_FACTOR/32 + 1] = a >> ((-int(SCALING_FACTOR)) & 0x1F);
    }
    if (a_negative) {
        fix_neg(r);
    }
}
