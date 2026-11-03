// Verify that consecutive RELR-eligible GOT entries are bitmap-packed.
//
// Four local symbols accessed via GOT produce consecutive GOT entries.
// With bitmap packing, they share one address entry and one bitmap entry
// instead of four individual RELR address entries.
// An absolute symbol's GOT entry must still be treated as a constant.
//
// Note: --no-relax is needed to prevent the linker from optimizing away
// the GOT references (REX_GOTPCRELX relaxation).
//#AbstractConfig:default
//#LinkArgs:-shared -z pack-relative-relocs --no-relax
//#LinkArgs:--defsym absolute=42
//#RunEnabled:false
//#ReferenceLinkers:lld
//#DiffIgnore:.dynamic.DT_FLAGS_1.NOW
//#DiffIgnore:section.got.plt.entsize

//#Config:x86_64:default
//#Arch:x86_64
//#ExpectSection:.relr.dyn max_entries=2

//#Config:android:x86_64
//#LinkArgs:--use-android-relr-tags

    .section .data,"aw",@progbits
    .type a, @object
a:  .quad 1
    .type b, @object
b:  .quad 2
    .type c, @object
c:  .quad 3
    .type d, @object
d:  .quad 4

    .section .text,"ax",@progbits
    .globl get_a
    .type get_a, @function
get_a:
    mov a@GOTPCREL(%rip), %rax
    ret

    .globl get_b
    .type get_b, @function
get_b:
    mov b@GOTPCREL(%rip), %rax
    ret

    .globl get_c
    .type get_c, @function
get_c:
    mov c@GOTPCREL(%rip), %rax
    ret

    .globl get_d
    .type get_d, @function
get_d:
    mov d@GOTPCREL(%rip), %rax
    ret

    .globl get_absolute
    .type get_absolute, @function
    .hidden absolute
get_absolute:
    mov absolute@GOTPCREL(%rip), %rax
    ret
