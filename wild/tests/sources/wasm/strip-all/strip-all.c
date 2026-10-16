//#AbstractConfig:base
//#CompArgs:-g
//#DiffEnabled:false

//#Config:default:base
//#ExpectSection:name
//#ExpectSection:target_features
//#ExpectSection:.debug_info
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

void strip_all_named_fn(void) {}

void _start(void) { strip_all_named_fn(); }
