//#Mode:static
//#Shared:force-dynamic-linking.c
//#ExpectError:(?i-u)Attempted static link of dynamic object
//#CompArgs:-fPIE
//#CompSoArgs:-fPIE

int main() { return 42; }
