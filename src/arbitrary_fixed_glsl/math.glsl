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

    r = sin_table[0];

    for (int i = 1; i < TRIG_PRECISION; ++i) {
        fix_mul(r, r, a);
        fix_add(r, r, sin_table[i]);
    }

    fix_mul(r, r, a);

    uint ONE[SIZE];
    fix_from_uint(ONE, 1);

    fix_add(r, r, ONE);

    if (!is_neg) {
        fix_neg(r);
    }
}

// This function computes 2*arctanh((a-1)/(a+1))
// Somebody much smarter than me worked out that it's equal to ln(a) 
void fix_ln(out uint r[SIZE], in uint a[SIZE]) {
    // find the most significant bit
    int i = int(SIZE) - 1;
    for (; i >= 0; --i) {
        if (a[i] != 0) {
            break;
        }
    }

    int msb = i*32 + findMSB(a[i]);
    // if the input is zero we should probably not try to take the log of it
    if (msb == -1) {
        r = FIX_ZERO;
        return;
    }

    // shift the input so as that it is between 0 and 1
    int offset = int(msb) - int(SCALING_FACTOR);
    if (offset > 0) {
        fix_rshift(a, a, offset);
    }
    else {
        fix_lshift(a, a, -offset);
    }

    // (a - 1)/(a + 1)
    fix_add(r, a, FIX_NEG_ONE);
    fix_add(a, a, FIX_ONE);
    fix_div(a, r, a);
    
    // need a^2 for efficient taylor series-ing
    uint a2[SIZE];
    fix_mul(a2, a, a);

    // Perform the taylor series of arctanh(x)
    r = log_table[0];
    for (int i = 0; i < LOG_PRECISION; ++i) {
        fix_mul(r, r, a2);
        fix_add(r, r, log_table[i]);
    }
    fix_mul(r, r, a);
  
    // Multiply result by 2
    fix_lshift1(r, r);
  
    // add offset to the result
    fix_from_int(a, offset);
    fix_mul(a, a, FIX_LN_2);
    fix_add(r, r, a);
}
