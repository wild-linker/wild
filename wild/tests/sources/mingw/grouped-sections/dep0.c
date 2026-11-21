__attribute__((section(".data"))) int dep0_ungrouped = 0;
__attribute__((section(".data$A"))) int dep0_grouped_a = 0;
__attribute__((section(".data$C"))) int dep0_grouped_c = 0;
__attribute__((section(".data$a"))) int dep0_grouped_lowercase_a = 0;
__attribute__((section(".data$c"))) int dep0_grouped_lowercase_c = 0;
__attribute__((section(".data$1"))) int dep0_grouped_1 = 0;
__attribute__((section(".data$3"))) int dep0_grouped_3 = 0;

__attribute__((section(".custom$A"))) int dep0_custom_a = 0;
__attribute__((section(".custom$C"))) int dep0_custom_c = 0;

__attribute__((section(".data$.very_long_group_priority_ending_with_a"))) int dep0_long_group_a = 0;
__attribute__((section(".data$.very_long_group_priority_ending_with_c"))) int dep0_long_group_c = 0;
