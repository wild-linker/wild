void missing_from_member(void);

int pulls_missing(void) {
  missing_from_member();
  return 1;
}

__attribute__((used)) void missing_kept(void) {}
