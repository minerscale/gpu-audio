bool fix_is_negative(in uint a[SIZE]) {
    return (a[SIZE - 1] & 0x80000000) > 0;
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

void fix_lshift(out uint r[SIZE], in uint a[SIZE], uint n) {
    for (int i = 0; i < n/32; ++i) {
        r[i] = 0;
    }

    r[n/32] = a[0] << (n & 0x1F);

    for (uint i = n/32 + 1; i < SIZE; ++i) {
        r[i] = (a[i - n/32] << (n & 0x1F)) | ((uint((n & 0x1F) != 0)) * (a[i - 1 - n/32] >> ((-n) & 0x1F)));
    }
}

void fix_rshift(out uint r[SIZE], in uint a[SIZE], uint n) {
    for (int i = 0; i < n/32; ++i) {
        r[SIZE - i - 1] = 0;
    }

    r[SIZE - 1 - (n/32)] = a[SIZE - 1] >> (n & 0x1F);

    for (int i = int(SIZE) - int(n/32) - 2; i >= 0; --i) {
        r[i] = ((uint((n & 0x1F) != 0)) * (a[i + 1 + n/32] << ((-n) & 0x1F))) | (a[i + n/32] >> (n & 0x1F));
    }
}

void fix_truncate(inout uint r[SIZE], in uint b) {
    for (int i = int(SIZE) - 1; i >= int(SIZE) - (b/32); --i) {
        r[i] = 0;
    }

    r[SIZE - 1 - b/32] &= 0xFFFFFFFF >> (b & 0x1F);
}
