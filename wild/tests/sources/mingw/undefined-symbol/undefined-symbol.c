//#Object:runtime.c
//#ExpectErrorWild: Undefined symbol missing_function

extern int missing_function(void);

int main(void) { return missing_function(); }
