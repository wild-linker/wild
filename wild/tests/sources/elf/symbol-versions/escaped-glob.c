void xfoobarx(void) {}
void preapost(void) {}
void prebpost(void) {}
void prexpost(void) {}
void clsa(void) {}
void clsb(void) {}
void cxxx(void) {}
void banga(void) {}
void bangb(void) {}
void dasha(void) {}
void dashm(void) {}
void dashz(void) {}

__asm__(
    ".globl \"pre*post\"\n"
    ".globl \"cls]\"\n"
    ".globl \"br]\"\n"
    ".globl \"bang!\"\n"
    ".globl \"dash-\"\n"
    ".globl \"only!\"\n"
    ".set \"pre*post\", preapost\n"
    ".set \"cls]\", clsa\n"
    ".set \"br]\", clsa\n"
    ".set \"bang!\", banga\n"
    ".set \"dash-\", dasha\n"
    ".set \"only!\", banga\n");
