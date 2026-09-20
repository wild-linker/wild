//#LinkerDriver:clang++

#include <iostream>

struct Foo {
  static int foo() { return 42; }
};

int main() {
  std::cout << "hello world\n" << std::endl;
  return Foo::foo();
}
