double CFAbsoluteTimeGetCurrent(void);

int main(void) { return CFAbsoluteTimeGetCurrent() > 0.0 ? 42 : 1; }
