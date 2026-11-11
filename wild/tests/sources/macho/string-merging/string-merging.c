//#AbstractConfig:base
//#Object:runtime.c
//#Object:string_merging1.s
//#Object:string_merging2.s
//#ReferenceLinkers:
//#DiffEnabled:false

//#Config:optimized:base
//#ExpectSectionBytes:__cstring=0x576f726c640048656c6c6f00
//#ExpectSectionBytes:__custom_cstring=0x4e6f207265666572656e636520746f207468697320737472696e670048656c6c6f00
//#ExpectSectionBytes:__literal_loc1=0x4c6f63616c3100
//#ExpectSectionBytes:__regular_data=0x416c69676e656400
//#NoSection:__empty_cstring

//#Config:disabled:base
//#LinkArgs:-O0
//#CompArgs:-DMERGING_DISABLED
//#ExpectSectionBytes:__cstring=0x48656c6c6f00576f726c6400576f726c640048656c6c6f00
//#ExpectSectionBytes:__custom_cstring=0x48656c6c6f004e6f207265666572656e636520746f207468697320737472696e670048656c6c6f00
//#ExpectSectionBytes:__literal_loc1=0x4c6f63616c3100
//#ExpectSectionBytes:__regular_data=0x416c69676e656400
//#ExpectSectionBytes:__empty_cstring=0x

#include "../common/runtime.h"

extern const char s1h[];
extern const char s2h[];
extern const char s3h[];
extern const char s4h[];
extern const char s1w[];
extern const char s2w[];
extern const char a1[];

const char* get_loc1(void);
const char* get_s1w(void);
const char* get_s2w(void);
const char* get_s2w_via_offset(void);

void main(void) {
#ifdef MERGING_DISABLED
  if (s1h == s2h) {
#else
  if (s1h != s2h) {
#endif
    exit_syscall(101);
  }
  if (s1h[0] != 'H') {
    exit_syscall(103);
  }
#ifdef MERGING_DISABLED
  if (s1w == s2w) {
#else
  if (s1w != s2w) {
#endif
    exit_syscall(102);
  }
  if (s1w[0] != 'W') {
    exit_syscall(103);
  }
  if (get_loc1()[0] != 'L') {
    exit_syscall(104);
  }
  if (a1[0] != 'A') {
    exit_syscall(105);
  }
#ifdef MERGING_DISABLED
  if (get_s1w() == get_s2w()) {
#else
  if (get_s1w() != get_s2w()) {
#endif
    exit_syscall(106);
  }
  if (get_s1w() != s1w) {
    exit_syscall(107);
  }
#ifdef MERGING_DISABLED
  if (s3h == s4h) {
#else
  if (s3h != s4h) {
#endif
    exit_syscall(108);
  }
  if (s3h == s1h) {
    exit_syscall(109);
  }
  if (s3h[0] != 'H') {
    exit_syscall(110);
  }
#ifdef MERGING_DISABLED
  if (get_s2w_via_offset() != s1w) {
#else
  if (get_s2w_via_offset() != get_s2w()) {
#endif
    exit_syscall(111);
  }
  exit_syscall(42);
}

//#Contains:No reference to this string
