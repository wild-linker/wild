//#AbstractConfig:base
//#CompArgs:-g
//#DiffEnabled:false

//#Config:default:base
//#ExpectSection:name
//#ExpectSection:target_features
// TODO(wasm): This should be `ExpectSection` and `ReferenceLinkers` should be removed when full
// debug info support lands
//#NoSection:.debug_info
//#ReferenceLinkers:
//#Contains:strip_all_named_fn

//#Config:strip-debug:base
//#LinkArgs:--strip-debug
//#ExpectSection:name
//#ExpectSection:target_features
//#NoSection:.debug_info
//#Contains:strip_all_named_fn

//#Config:strip-all:base
//#LinkArgs:--strip-all
//#NoSection:name
//#NoSection:target_features
//#NoSection:.debug_info
//#DoesNotContain:strip_all_named_fn

//#Config:strip-all-short:base
//#LinkArgs:-s
//#NoSection:name
//#NoSection:target_features
//#NoSection:.debug_info

//#Config:strip-all-then-debug:base
//#LinkArgs:--strip-all --strip-debug
//#NoSection:name
//#NoSection:target_features
//#NoSection:.debug_info

//#Config:keep-target-features:base
//#LinkArgs:--strip-all --keep-section=target_features
//#NoSection:name
//#ExpectSection:target_features
//#NoSection:.debug_info
//#DoesNotContain:strip_all_named_fn

//#Config:keep-name:base
//#LinkArgs:--strip-all --keep-section=name
//#ExpectSection:name
//#NoSection:target_features
//#NoSection:.debug_info
//#Contains:strip_all_named_fn

//#Config:keep-both:base
//#LinkArgs:--strip-all --keep-section=name --keep-section=target_features
//#ExpectSection:name
//#ExpectSection:target_features
//#NoSection:.debug_info
//#Contains:strip_all_named_fn

//#Config:keep-debug-ignored:base
//#LinkArgs:--strip-all --keep-section=.debug_info
//#NoSection:name
//#NoSection:target_features
//#NoSection:.debug_info

//#Config:strip-all-short-keep-features:base
//#LinkArgs:-s --keep-section=target_features
//#NoSection:name
//#ExpectSection:target_features
//#NoSection:.debug_info

void strip_all_named_fn(void) {}

void _start(void) { strip_all_named_fn(); }
