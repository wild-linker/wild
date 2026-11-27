.section .idata$4,"dr"
.rva hn_IsDebuggerPresent
.long 0

.section .idata$5,"dr"
.globl __imp_IsDebuggerPresent
__imp_IsDebuggerPresent:
.rva hn_IsDebuggerPresent
.long 0

.section .idata$6,"dr"
hn_IsDebuggerPresent:
.short 0
.asciz "IsDebuggerPresent"
.section .idata$7,"dr"
.rva _head_libfoo_a
