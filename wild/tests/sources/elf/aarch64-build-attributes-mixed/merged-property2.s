// Use two notes in an over-aligned section to verify that note normalization
// removes the extra inter-note padding and continues parsing the next note.
.section ".note.gnu.property", "a", @note
.p2align 4

// First note: PAuth. Its 24-byte descriptor ends at an offset that requires
// extra padding for the 16-byte section alignment.
.long 4
.long pauth_end - pauth_begin
.long 5
.asciz "GNU"
.p2align 4
pauth_begin:
.long 0xc0000001
.long 16
.quad 0x31
.quad 0x13
pauth_end:

.p2align 4

// Second note: BTI | PAC. Reaching this note requires advancing past the
// source-alignment padding above.
.long 4
.long feature_end - feature_begin
.long 5
.asciz "GNU"
.p2align 4
feature_begin:
.long 0xc0000000
.long 4
.long 3
.long 0
feature_end:
