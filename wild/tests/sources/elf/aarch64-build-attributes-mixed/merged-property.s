.section ".note.gnu.property", "a"
.long 4
.long end - begin
.long 5
.asciz "GNU"
.p2align 3
begin:
.long 0xc0000000
.long 4
.long 7
.long 0

.long 0xc0000001
.long 16
.quad 0x31
.quad 0x13
.p2align 3
end:
