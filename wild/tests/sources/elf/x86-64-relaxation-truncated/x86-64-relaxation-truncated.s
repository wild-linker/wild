// Checks that a relaxation is only applied if the section is long enough for everything the
// relaxation rewrites. Each input recognisably starts an instruction sequence, but is cut short of
// the rewritten bytes. These used to panic.
//#AbstractConfig:default
//#Arch:x86_64
//#LinkArgs:-static
// lld accepts the tlsdesc-call and jmp-indirect inputs, so only GNU ld is a reference.
//#ReferenceLinkers:bfd

// GNU ld and lld reject these TLSLD sequences, while Wild declines the relaxation and links, so
// there's no reference linker and nothing to run.
//#AbstractConfig:no-relaxation:default
//#ReferenceLinkers:
//#RunEnabled:false

//#Config:tlsld:no-relaxation
//#Object:tlsld.s

//#Config:tlsld64:no-relaxation
//#Object:tlsld64.s

//#Config:tlsld-noplt:no-relaxation
//#Object:tlsld-noplt.s

//#Config:tlsdesc-call:default
//#Object:tlsdesc-call.s
//#ExpectError:TLSDESC

//#Config:jmp-indirect:default
//#Object:jmp-indirect.s
//#ExpectError:outside of bounds|bad reloc offset

.section .note.GNU-stack,"",@progbits
