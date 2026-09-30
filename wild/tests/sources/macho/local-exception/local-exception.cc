//#LinkerDriver:clang++
//#Object:personality.cc
//#ExpectSym:___gxx_personality_v0 section="__text"
//#NoSection:__eh_frame
//#ExpectInstructions:__unwind_info 16 .long 1

int main() {
  try {
    throw 1;
  } catch (int value) {
    return value;
  }
}
