//#AbstractConfig:default
//#Arch:x86_64
//#ReferenceLinkers:bfd,lld
//#RunEnabled:false

//#Config:static:default
//#LinkArgs:--no-gc-sections
//#ExpectSectionBytes:test=0xb819000000b81b000000b819000000b81b000000
//#ExpectSectionBytes:.data=0x19000000000000001b0000000000000019000000000000001b00000000000000

//#Config:shared:default
//#LinkArgs:--no-gc-sections -shared
//#DiffIgnore:.dynamic.DT_FLAGS_1.NOW

.globl _start
_start:
    ret

.globl foo
.type foo,@object
.size foo,26
foo:
    .zero 26



.globl foohidden
.hidden foohidden
.type foohidden,@object
.size foohidden,26
foohidden:

.section test,"aw"
movl $foo@SIZE-1,%eax
movl $foo@SIZE+1,%eax
movl $foohidden@SIZE-1,%eax
movl $foohidden@SIZE+1,%eax

.data
.quad foo@SIZE-1
.quad foo@SIZE+1
.quad foohidden@SIZE-1
.quad foohidden@SIZE+1
