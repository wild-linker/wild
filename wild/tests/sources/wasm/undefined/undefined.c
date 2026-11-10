//#AbstractConfig:base
//#Archive:undefined-member.c,undefined-other.c,undefined-data.c,undefined-missing.c,undefined-weak.c

//#AbstractConfig:pulled:base
//#Contains:member_kept
//#Contains:other_kept
//#DoesNotContain:pulled_fn
//#DoesNotContain:member_dead
//#DoesNotContain:other_dead
//#DoesNotContain:other_fn
//#DoesNotContain:data_kept
//#DoesNotContain:missing_kept
//#DoesNotContain:weak_kept
//#NoSym:pulled_fn

//#Config:pull:pulled
//#LinkArgs:-u pulled_fn

//#Config:joined:pulled
//#LinkArgs:-upulled_fn

//#Config:long-eq:pulled
//#LinkArgs:--undefined=pulled_fn

//#Config:long-space:pulled
//#LinkArgs:--undefined pulled_fn

//#Config:absent:base
//#DoesNotContain:member_kept
//#DoesNotContain:other_kept
//#DoesNotContain:data_kept
//#DoesNotContain:missing_kept
//#DoesNotContain:weak_kept

//#Config:equals-short:base
//#LinkArgs:-u=pulled_fn
//#DoesNotContain:member_kept

//#Config:unknown:base
//#LinkArgs:-u does_not_exist
//#DoesNotContain:member_kept
//#DoesNotContain:weak_kept

//#Config:data:base
//#LinkArgs:-u pulled_data
//#Contains:data_kept
//#DoesNotContain:member_kept
//#DoesNotContain:other_kept
//#DoesNotContain:missing_kept
//#DoesNotContain:weak_kept

// The undefined call is in a function GC removes, so the link succeeds.
//#Config:dead-undef:base
//#LinkArgs:-u pulls_missing
//#Contains:missing_kept
//#DoesNotContain:pulls_missing
//#DoesNotContain:member_kept
//#DoesNotContain:other_kept

//#Config:dead-undef-no-gc:base
//#LinkArgs:-u pulls_missing --no-gc-sections
//#ExpectError:undefined symbol: missing_from_member

//#Config:live-undef:base
//#CompArgs:-DLIVE_UNDEFINED
//#LinkArgs:-u missing_from_main
//#ExpectError:undefined symbol: missing_from_main

//#Config:no-gc:base
//#LinkArgs:-u pulled_fn --no-gc-sections
//#Contains:pulled_fn
//#Contains:member_dead
//#Contains:member_kept
//#Contains:other_fn
//#Contains:other_dead
//#Contains:other_kept
//#DoesNotContain:data_kept
//#DoesNotContain:missing_kept
//#DoesNotContain:pulls_missing
//#DoesNotContain:weak_kept
//#NoSym:pulled_fn

// A weak reference does not load the member. `-u` does, and the reference keeps `weak_fn`.
//#Config:weak:base
//#LinkArgs:-u weak_fn
//#Contains:weak_kept
//#Contains:weak_fn
//#DoesNotContain:member_kept
//#NoSym:weak_fn

#ifdef LIVE_UNDEFINED
void missing_from_main(void);
#endif

__attribute__((weak)) void weak_fn(void);

void _start(void) {
#ifdef LIVE_UNDEFINED
  missing_from_main();
#endif
  if (weak_fn) {
    weak_fn();
  }
}
