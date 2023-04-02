// find most significant bit, get closest two bit aligned value and set value bitshifted right by x/2 as initial guess
// Only takes +ve numbers
void fix_sqrt(out uint r[SIZE], in uint a[SIZE]) {
    uint tmp[SIZE];
    
    r = FIX_ZERO;

    // find the most significant bit
    int i = int(SIZE) - 1;
    for (; i >= 0; --i) {
        if (a[i] != 0) {
            break;
        }
    }

    int msb = i*32 + findMSB(a[i]);
    // if the input is zero we should probably avoid a division
    if (msb == -1) {
        return;
    }

    uint guess = uint(((msb - int(SCALING_FACTOR))/2) + int(SCALING_FACTOR));

    // Set the guess
    r[guess/32] = 1 << (guess & 0x1F);

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
void fix_sin(out uint r[SIZE], in uint a[SIZE]) { 
    fix_rem(r, a, FIX_2_PI);
    fix_sub(r, r, FIX_PI);
    bool is_neg = fix_is_negative(r);
    fix_rem(a, a, FIX_PI);
    fix_sub(a, a, FIX_PI_2);
    fix_mul(a, a, a);

    r = sin_table[0];

    for (int i = 1; i < TRIG_PRECISION; ++i) {
        fix_mul(r, r, a);
        fix_add(r, r, sin_table[i]);
    }

    fix_mul(r, r, a);

    fix_add(r, r, FIX_ONE);

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
  
    // add offset to the result
    fix_from_int(a, offset);
    fix_mul(a, a, FIX_LN_2);
    fix_add(r, r, a);
}
