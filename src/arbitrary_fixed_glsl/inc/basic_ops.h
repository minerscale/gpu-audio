void fix_add(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]);
void fix_neg(inout uint r[SIZE]);
void fix_sub(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]);
void fix_mul(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]);
void fix_div(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]);
uint fix_div_by_u32(out uint r[SIZE], in uint a[SIZE], in uint b);
void fix_rem(out uint r[SIZE], in uint a[SIZE], in uint b[SIZE]);
void fix_floor(inout uint r[SIZE]);
