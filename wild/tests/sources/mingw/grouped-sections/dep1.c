__attribute__((section(".data"))) int dep1_ungrouped = 0;
__attribute__((section(".data$B"))) int dep1_grouped_b = 0;
__attribute__((section(".data$b"))) int dep1_grouped_lowercase_b = 0;
__attribute__((section(".data$2"))) int dep1_grouped_2 = 0;

__attribute__((section(".custom$B"))) int dep1_custom_b = 0;

__attribute__((section(".data$.very_long_group_priority_ending_with_b"))) int dep1_long_group_b = 0;
