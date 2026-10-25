//#AbstractConfig:base
//#RunEnabled:false
//#DiffEnabled:false
//#ReferenceLinkers:bfd
//#SkipArch:riscv64

//#Config:addr-alias:base
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-T ./addr-alias.ld
//#ExpectSym:after_addr section=".text",address=0x10010
//#ExpectSym:section_addr section=".text",address=0x10000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:min-max-number:base
//#LinkArgs:-T ./min-max-number.ld
//#ExpectSym:min_right section=".text",address=0x10000
//#ExpectSym:min_left section=".text",address=0x10000
//#ExpectSym:max_right section=".text",address=0x10010
//#ExpectSym:max_left section=".text",address=0x10010
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:cross-section-offsets:base
//#LinkArgs:-T ./cross-section-offsets.ld
//#ExpectSym:cross_min section=".data",address=0x20000
//#ExpectSym:cross_mul section=".data",address=0x20000
//#ExpectSym:cross_neg section=".data",address=0x20000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:nested-subtraction:base
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-T ./nested-subtraction.ld
//#ExpectSym:difference is_abs=true,address=0x20
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:logical-section-start:base
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-T ./logical-section-start.ld
//#ExpectSym:start_and is_abs=true,address=0x1
//#ExpectSym:start_or is_abs=true,address=0x1
//#ExpectSym:start_and_zero is_abs=true,address=0x0
//#ExpectSym:start_or_one is_abs=true,address=0x1
//#ExpectSym:start_and_expr is_abs=true,address=0x1
//#ExpectSym:start_or_expr is_abs=true,address=0x1
//#ExpectSym:start_not_expr is_abs=true,address=0x1
//#ExpectSym:start_not_expr_false is_abs=true,address=0x0
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:logical-mixed-operands:base
//#LinkArgs:-T ./logical-mixed-operands.ld
//#ExpectSym:mixed_and section=".text",address=0x10001
//#ExpectSym:mixed_or section=".text",address=0x10001
//#ExpectSym:mixed_nor section=".text",address=0x10000
//#ExpectSym:in_sec_nor is_abs=true,address=0x0
//#ExpectSym:out_sec_nor is_abs=true,address=0x0
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:negative-section-offsets:base
//#LinkArgs:-T ./negative-section-offsets.ld
//#ExpectSym:before_start section=".text",address=0xffff
//#ExpectSym:restored_start section=".text",address=0x10000
//#ExpectSym:minimum section=".text",address=0x10000
//#ExpectSym:in_sec_and section=".text",address=0x10001
//#ExpectSym:in_sec_zero_and section=".text",address=0x10000
//#ExpectSym:in_sec_not section=".text",address=0x10001
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:min-max-location:base
//#LinkArgs:-T ./min-max-location.ld
//#ExpectSym:padded_end section=".text",address=0x10100
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:absolute-arithmetic:base
//#ReferenceLinkers:bfd,lld
//#LinkArgs:-T ./absolute-arithmetic.ld
//#ExpectSym:product is_abs=true,address=0x20000
//#ExpectSym:quotient is_abs=true,address=0x8000
//#ExpectSym:shifted is_abs=true,address=0x8000
//#ExpectSym:reverse_difference is_abs=true,address=0x10000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:relative-plus-absolute:base
//#LinkArgs:-T ./relative-plus-absolute.ld
//#ExpectSym:absolute_offset is_abs=true,address=0x10
//#ExpectSym:derived is_abs=true,address=0x10010
//#ExpectSym:derived_reverse is_abs=true,address=0x10010
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:unaligned-align:base
//#LinkArgs:-T ./unaligned-align.ld
//#ExpectSym:unaligned_dot_align section=".text",address=0x10010
//#ExpectSym:unaligned_expr_align section=".text",address=0x10004
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:expression-arith:base
//#LinkArgs:-T ./expression-arith.ld
//#ExpectSym:in_sec_add section=".text",address=0x10010
//#ExpectSym:in_sec_sub section=".text",address=0x10010
//#ExpectSym:in_sec_mul section=".text",address=0x10020
//#ExpectSym:in_sec_add_rev section=".text"
//#ExpectSym:in_sec_sub_rev section=".text"
//#ExpectSym:in_sec_loadaddr_sub is_abs=true,address=0xf000
//#ExpectSym:in_sec_loadaddr_mul is_abs=true,address=0x20000
//#ExpectSym:in_sec_loadaddr_and is_abs=true,address=0x0
//#ExpectSym:in_sec_loadaddr_diff section=".text",address=0xff00
//#ExpectSym:in_sec_loadaddr_diff_rev section=".text",address=0x10100
//#ExpectSym:in_sec_loadaddr_add section=".text",address=0x30100
//#ExpectSym:in_sec_cross_diff section=".text",address=0x20000
//#ExpectSym:in_sec_cross_add section=".text",address=0x40000
//#ExpectSym:in_sec_cross_ref section=".data",address=0x20000
//#ExpectSym:in_sec_neg section=".text",address=0x10000
//#ExpectSym:in_sec_not section=".text",address=0xffff
//#ExpectSym:in_sec_abs_alias section=".text",address=0x10020
//#ExpectSym:in_sec_align section=".text",address=0x10000
//#ExpectSym:in_sec_align_expr section=".text",address=0x10100
//#ExpectSym:mid_align section=".data",address=0x20200
//#ExpectSym:out_add is_abs=true,address=0x11000
//#ExpectSym:out_sub is_abs=true,address=0xf000
//#ExpectSym:out_sub_rev is_abs=true,address=0x20000
//#ExpectSym:out_mul is_abs=true,address=0x20000
//#ExpectSym:out_div is_abs=true,address=0x8000
//#ExpectSym:out_shift is_abs=true,address=0x8000
//#ExpectSym:out_diff is_abs=true,address=0x100
//#ExpectSym:out_rel_plus_abs is_abs=true,address=0x10020
//#ExpectSym:out_loadaddr_and is_abs=true,address=0x0
//#ExpectSym:out_loadaddr_diff_rev is_abs=true,address=0x100
//#ExpectSym:out_loadaddr_add is_abs=true,address=0x20100
//#ExpectSym:out_cross_diff is_abs=true,address=0x10000
//#ExpectSym:out_cross_add is_abs=true,address=0x30000
//#ExpectSym:out_neg section=".text",address=0x10000
//#ExpectSym:out_not section=".text",address=0xffff
//#ExpectSym:out_abs_alias is_abs=true,address=0x20
//#ExpectSym:out_align_expr is_abs=true,address=0x10100
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:min-max-address:base
//#LinkArgs:-T ./min-max-address.ld
//#ExpectSym:min_seg section=".text",address=0x10000
//#ExpectSym:max_seg section=".text",address=0x10020
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

//#Config:segment-start-operations:base
//#LinkArgs:-T ./segment-start-operations.ld
//#ExpectSym:sym_mul section=".text",address=0x10000
//#ExpectSym:sym_mul_offset section=".text",address=0x10040
//#ExpectSym:sym_shl section=".text",address=0x10000
//#ExpectSym:sym_and section=".text",address=0x10000
//#ExpectSym:sym_orig_mul section=".text",address=0x10000
//#ExpectSym:neg_seg section=".text",address=0x10000
//#ExpectSym:neg_seg_offset section=".text",address=0xffe0
//#ExpectSym:neg_orig section=".text",address=0x10000
//#CompArgs:-fPIE
//#LinkArgs:-no-pie

int data_var = 42;

__attribute__((aligned(0x1))) void _start(void) {}
