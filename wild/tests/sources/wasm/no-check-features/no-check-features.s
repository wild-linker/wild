//#Object:no-check-features-disallow.s
//#ExpectError:disallowed

//#Config:no-check
//#Object:no-check-features-disallow.s
//#LinkArgs:--no-check-features
//#ExpectSection:target_features
//#Contains:foobar

//#Config:restore-check
//#Object:no-check-features-disallow.s
//#LinkArgs:--no-check-features --check-features
//#ExpectError:disallowed

  .globl _start
_start:
  .functype _start () -> ()
  end_function

  .section .custom_section.target_features,"",@
  .int8 1 # Number of target features
  .int8 43 # '+'
  .int8 6 # Length of "foobar"
  .ascii "foobar"
