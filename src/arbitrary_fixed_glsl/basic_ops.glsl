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

void fix_neg(inout uint r[SIZE]) {
    bool carry_prev = true;
    for (uint i = 0; i < SIZE; ++i) {
        r[i] = ~r[i];
        // add overflow
        r[i] += uint(carry_prev);
        // detect overflow on carry
        carry_prev = carry_prev && r[i] == 0;
    }
}

void fix_sub(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    fix_neg(b);
    fix_add(r, a, b);
}

// !!TODO!! find a less shitty algorithm that isn't O(n^2)
void fix_mul(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    uint res[2*SIZE];

    for (int i = 0; i < 2*SIZE; ++i) {
        res[i] = 0;
    }

    bool a_is_negative = fix_is_negative(a);
    bool b_is_negative = fix_is_negative(b);

    if (a_is_negative) {
        fix_neg(a);
    }

    if (b_is_negative) {
        fix_neg(b);
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
        fix_neg(r);
    }
}

void fix_div(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]) {
    bool a_is_negative = fix_is_negative(a);
    bool b_is_negative = fix_is_negative(b);

    if (a_is_negative) {
        fix_neg(a);
    }

    if (b_is_negative) {
        fix_neg(b);
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
        rem_d[0] = res[0] << 1;
        for (int i = 1; i < 2*SIZE; ++i) {
            rem_d[i] = (res[i] << 1) | (res[i - 1] >> 31);
        }

        res = rem_d;

        bool carry_prev = true;
        for (uint i = 0; i < 2*SIZE; ++i) {
            rem_d[i] = ~D[i];
            // add overflow
            rem_d[i] += uint(carry_prev);
            // detect overflow on carry
            carry_prev = carry_prev && rem_d[i] == 0;
        }

        carry_prev = false;

        for (uint i = 0; i < 2*SIZE; ++i) {
            rem_d[i] = rem[i] + rem_d[i];
            
            // detect overflow
            bool carry = (rem_d[i] < rem[i]);
            // add previous overflow
            rem_d[i] += uint(carry_prev);
            // detect overflow on carry
            carry = carry || (carry_prev && rem_d[i] == 0);

            carry_prev = carry;
        }

        if ((rem_d[2 * SIZE - 1] & 0x80000000) == 0) {
            res[0] |= 1;
            rem = rem_d;
        }

        rem_d[2*SIZE - 1] = D[2*SIZE - 1] >> 1;
        for (int i = 2*int(SIZE) - 2; i >= 0; --i) {
            rem_d[i] = (D[i + 1] << 31) | (D[i] >> 1);
        }

        D = rem_d;
    }

    // Shift back into place
    for (int i = 0; i < SIZE; ++i) {
        r[i] = (((SCALING_FACTOR & 0x1F) > 0)?(res[i + SIZE - 1 - (SCALING_FACTOR/32)] >> ((-SCALING_FACTOR) & 0x1F)):0) |
               (res[i + SIZE - (SCALING_FACTOR/32)] << ((SCALING_FACTOR & 0x1F)));
    }

    if (a_is_negative != b_is_negative) {
        fix_neg(r);
    }
}

uint fix_div_by_u32(out uint r[SIZE], in uint a[SIZE], in uint b) {
    bool a_is_negative = fix_is_negative(a);

    if (a_is_negative) {
        fix_neg(a);
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
        fix_neg(r);
    }

    return uint(temp);
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
