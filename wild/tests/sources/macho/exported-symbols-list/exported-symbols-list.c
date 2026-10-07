//#LinkerDriver:clang
//#LinkArgs:-dynamiclib -Wl,-exported_symbols_list,./exported-symbols-list.txt
//#RunEnabled:false
//#Archive:unreferenced.c
//#ExpectDynSym:_wild_test_export_exact
//#ExpectDynSym:_wild_test_export_glob_one
//#ExpectDynSym:_wild_test_export_prefix
//#ExpectDynSym:_leading_wild_test_export_prefix
//#ExpectDynSym:_wild_test_export_midend
//#ExpectDynSym:_wild_test_export_mid_extra_end
//#ExpectDynSym:_wild_test_export_class_*_tail
//#ExpectDynSym:_wild_test_export_question_x
//#ExpectDynSym:_wild_test_export_range_b
//#NoDynSym:_wild_test_export_not_exported
//#NoDynSym:_wild_test_export_prefix_other
//#NoDynSym:_wild_test_export_mid_extra_other
//#NoDynSym:_wild_test_export_class_x_tail
//#NoDynSym:_wild_test_export_question_xy
//#NoDynSym:_wild_test_export_range_z
//#NoSym:_wild_test_export_archive_unreferenced
//#NoDynSym:_wild_test_private_data
//#ExpectSym:_wild_test_private_data section="__private",offset-in-section=0,binding=local

int wild_test_export_exact(void) { return 1; }
int wild_test_export_glob_one(void) { return 2; }
int wild_test_export_prefix(void) { return 2; }
int leading_wild_test_export_prefix(void) { return 2; }
int wild_test_export_midend(void) { return 2; }
int wild_test_export_mid_extra_end(void) { return 2; }
int wild_test_export_prefix_other(void) { return 2; }
int wild_test_export_mid_extra_other(void) { return 2; }
int wild_test_export_question_x(void) { return 3; }
int wild_test_export_question_xy(void) { return 3; }
int wild_test_export_range_b(void) { return 4; }
int wild_test_export_range_z(void) { return 4; }
int wild_test_export_not_exported(void) { return 5; }
__attribute__((section("__DATA,__private"))) int wild_test_private_data = 6;
int class_star __asm__("_wild_test_export_class_*_tail") = 7;
int wild_test_export_class_x_tail = 8;
