//#LinkArgs:--features=foobar,relaxed-simd
//#ExpectSection:target_features
//#Contains:foobar
//#Contains:relaxed-simd

//#Config:not-allowed
//#LinkArgs:--features=relaxed-simd
//#ExpectError:not allowed

//#Config:no-check
//#LinkArgs:--no-check-features --features=relaxed-simd
//#ExpectSection:target_features
//#Contains:relaxed-simd
//#DoesNotContain:foobar

//#Config:empty
//#LinkArgs:--features=
//#ExpectError:not allowed

//#Config:disallow
//#LinkArgs:--features=-foobar
//#ExpectError:not allowed

//#Config:prefix
//#LinkArgs:--features=+foobar,+relaxed-simd
//#ExpectError:not allowed

//#Config:literal
//#LinkArgs:--no-check-features --features=+foobar,+relaxed-simd
//#ExpectSection:target_features
//#Contains:+foobar
//#Contains:+relaxed-simd

//#Config:extra
//#LinkArgs:--features=foobar --extra-features=relaxed-simd
//#ExpectSection:target_features
//#Contains:foobar
//#Contains:relaxed-simd

  .globl _start
_start:
  .functype _start () -> ()
  end_function

  .section .custom_section.target_features,"",@
  .int8 1
  .int8 43
  .int8 6
  .ascii "foobar"
