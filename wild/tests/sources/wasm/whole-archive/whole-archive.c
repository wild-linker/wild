//#AbstractConfig:base
//#Archive:whole-archive-used.c,whole-archive-unused.c
//#ExpectSection:name
//#Contains:used_from_archive

//#Config:default:base
//#CompArgs:-DEXPECT_CTOR=0
//#DoesNotContain:unused_kept_fn
//#DoesNotContain:unused_dead_fn

//#Config:whole:base
//#LinkArgs:--whole-archive
//#CompArgs:-DEXPECT_CTOR=1
//#Contains:unused_kept_fn
//#DoesNotContain:unused_dead_fn

//#Config:whole-no-gc:base
//#LinkArgs:--whole-archive --no-gc-sections
//#CompArgs:-DEXPECT_CTOR=1
//#Contains:unused_kept_fn
//#Contains:unused_dead_fn

//#Config:restore:base
//#LinkArgs:--whole-archive --no-whole-archive
//#CompArgs:-DEXPECT_CTOR=0
//#DoesNotContain:unused_kept_fn
//#DoesNotContain:unused_dead_fn

// The flag applies only to the archive that follows it. The later archive is not
// loaded, so its undefined symbol is not an error.
//#Config:scoped
//#CompArgs:-DEXPECT_CTOR=1
//#Archive:template(--whole-archive $O --no-whole-archive):whole-archive-used.c,whole-archive-unused.c
//#Archive:whole-archive-missing.c
//#Contains:unused_kept_fn
//#DoesNotContain:unused_dead_fn

// A later `--whole-archive` does not apply to an earlier archive.
//#Config:not-preceding
//#CompArgs:-DEXPECT_CTOR=0
//#Archive:whole-archive-used.c,whole-archive-unused.c
//#Archive:template($O --whole-archive):whole-archive-missing.c
//#DoesNotContain:unused_kept_fn
//#DoesNotContain:unused_dead_fn

//#Config:via-lib
//#LinkArgs:--whole-archive
//#PostLinkArgs:-L$OUT_DIR -lwholemembers
//#CompArgs:-DEXPECT_CTOR=1
//#Archive:as(libwholemembers.a),noadd:whole-archive-used.c,whole-archive-unused.c
//#Contains:unused_kept_fn
//#DoesNotContain:unused_dead_fn

//#Config:undefined:base
//#LinkArgs:--whole-archive
//#Archive:whole-archive-missing.c
//#ExpectError:undefined symbol: missing_from_whole_archive

#ifndef EXPECT_CTOR
#define EXPECT_CTOR 0
#endif

extern int used_from_archive(void);

int side_effect;

void _start(void) {
  if (used_from_archive() != 7) {
    __builtin_trap();
  }
  if (side_effect != EXPECT_CTOR) {
    __builtin_trap();
  }
}
