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

// TODO, need a pi constant not precise enough
// TODO, also 1 const
void fix_cos(out uint r[SIZE], in uint a[SIZE]) { 
    fix_lshift1(r, FIX_PI);
    fix_rem(r, a, r);
    fix_sub(r, r, FIX_PI);
    bool is_neg = fix_is_negative(r);
    fix_rem(a, a, FIX_PI);
    fix_mul(a, a, a);

    r = taylor_table[0];

    for (int i = 1; i < TRIG_PRECISION; ++i) {
        fix_mul(r, r, a);
        fix_add(r, r, taylor_table[i]);
    }

    fix_mul(r, r, a);

    uint ONE[SIZE];
    fix_from_uint(ONE, 1);

    fix_add(r, r, ONE);

    if (!is_neg) {
        fix_neg(r);
    }
}

// shift a until in range of 1 to 2
// Taylor series go brrrrrr
void fix_ln(out uint r[SIZE], in uint a[SIZE]) {
    // TODO
}
