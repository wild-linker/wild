void missing_from_whole_archive(void);

__attribute__((constructor)) static void call_missing(void) { missing_from_whole_archive(); }
