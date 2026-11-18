//#AbstractConfig:default
//#Object:runtime.c
//#Object:dep0.c
//#Object:dep1.c
//#ReferenceLinkers:bfd,lld

//#Config:regular_sections:default
//#Variant:1

//#Config:custom_sections:default
//#Variant:2

//#Config:long_group_name:default
//#Variant:3

extern int dep0_ungrouped;
extern int dep0_grouped_a;
extern int dep0_grouped_c;
extern int dep0_grouped_lowercase_a;
extern int dep0_grouped_lowercase_c;
extern int dep0_grouped_1;
extern int dep0_grouped_3;
extern int dep1_ungrouped;
extern int dep1_grouped_b;
extern int dep1_grouped_lowercase_b;
extern int dep1_grouped_2;

extern int dep0_custom_a;
extern int dep0_custom_c;
extern int dep1_custom_b;

extern int dep0_long_group_a;
extern int dep0_long_group_c;
extern int dep1_long_group_b;

int main() {
#if VARIANT == 1
  if (&dep0_ungrouped > &dep1_ungrouped) {
    return 0;
  }
  if (&dep1_ungrouped > &dep0_grouped_1) {
    return 1;
  }
  if (&dep0_grouped_1 > &dep1_grouped_2) {
    return 2;
  }
  if (&dep1_grouped_2 > &dep0_grouped_3) {
    return 3;
  }
  if (&dep0_grouped_3 > &dep0_grouped_a) {
    return 4;
  }
  if (&dep0_grouped_a > &dep1_grouped_b) {
    return 5;
  }
  if (&dep1_grouped_b > &dep0_grouped_c) {
    return 6;
  }
  if (&dep0_grouped_c > &dep0_grouped_lowercase_a) {
    return 7;
  }
  if (&dep0_grouped_lowercase_a > &dep1_grouped_lowercase_b) {
    return 8;
  }
  if (&dep1_grouped_lowercase_b > &dep0_grouped_lowercase_c) {
    return 9;
  }
  return 42;
#endif

#if VARIANT == 2
  if (&dep0_custom_a > &dep1_custom_b) {
    return 10;
  }
  if (&dep1_custom_b > &dep0_custom_c) {
    return 11;
  }
  return 42;
#endif

#if VARIANT == 3
  if (&dep0_long_group_a > &dep1_long_group_b) {
    return 20;
  }
  if (&dep1_custom_b > &dep0_custom_c) {
    return 21;
  }
  return 42;
#endif

  return -1;
}
