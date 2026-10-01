#!/usr/bin/env bash
# Generates the small-link regression captures used by HOW_TO_PR_UPSTREAM.md:
#
#   tinyc   5,000 one-function C files plus a main that calls them all (many tiny debug sections)
#   cxxdbg  1,500 C++ files using std::string/vector/map plus a main (moderate debug info)
#
# Both are compiled with clang at -O1 with DWARF 5 debug info, one function/data section per
# symbol, then linked once by the given Wild binary under WILD_SAVE_BASE, leaving a save-dir capture
# that capture_ab.py can relink.
#
# usage: make_small_captures.sh <wild binary> <output dir>
#   -> <output dir>/cap-tinyc and <output dir>/cap-cxxdbg (each contains run-with)
set -euo pipefail

wild=$(realpath "$1")
out=$(realpath -m "$2")
jobs=$(nproc)
cflags=(-c -O1 -g -gdwarf-5 -ffunction-sections -fdata-sections)

mkdir -p "$out"

make_tinyc() {
  local dir=$out/tinyc
  mkdir -p "$dir/src" "$dir/obj"
  {
    for i in $(seq 0 4999); do echo "int f$i(int);"; done
    echo "int main(int c, char **v) { int s = 0;"
    for i in $(seq 0 4999); do echo "  s += f$i(c);"; done
    echo "  return s & 1; }"
  } > "$dir/src/main.c"
  for i in $(seq 0 4999); do
    printf 'int g%d;\nint f%d(int x) { return x * %d + g%d; }\n' "$i" "$i" "$i" "$i" > "$dir/src/t$i.c"
  done
  ls "$dir"/src/*.c | xargs -P "$jobs" -I{} sh -c \
    'clang '"${cflags[*]}"' "$1" -o "$2/obj/$(basename "$1" .c).o"' _ {} "$dir"
  rm -rf "$out/cap-tinyc-all"
  WILD_SAVE_BASE="$out/cap-tinyc-all" clang --ld-path="$wild" "$dir"/obj/*.o -o "$dir/tinyc.bin"
  ln -sfn "$out/cap-tinyc-all/0" "$out/cap-tinyc"
}

make_cxxdbg() {
  local dir=$out/cxxdbg
  mkdir -p "$dir/src" "$dir/obj"
  for i in $(seq 0 1499); do
    cat > "$dir/src/f$i.cpp" <<EOF
#include <string>
#include <vector>
#include <map>
namespace m$i {
struct Item$i { std::string name; int value; double weight; };
static std::map<std::string, int> table$i;
int work$i(int n) {
  std::vector<Item$i> v;
  for (int k = 0; k < n; ++k) v.push_back(Item$i{"item" + std::to_string(k), k * $i, k * 0.5});
  int s = 0;
  for (auto &it : v) { s += it.value + (int)it.name.size(); table$i[it.name] = s; }
  return s;
}
}
int entry$i(int n) { return m$i::work$i(n); }
EOF
  done
  {
    for i in $(seq 0 1499); do echo "int entry$i(int);"; done
    echo "int main(int argc, char**) { int s = 0;"
    for i in $(seq 0 1499); do echo "  s += entry$i(argc);"; done
    echo "  return s & 1; }"
  } > "$dir/src/main.cpp"
  ls "$dir"/src/*.cpp | xargs -P "$jobs" -I{} sh -c \
    'clang++ '"${cflags[*]}"' "$1" -o "$2/obj/$(basename "$1" .cpp).o"' _ {} "$dir"
  rm -rf "$out/cap-cxxdbg-all"
  WILD_SAVE_BASE="$out/cap-cxxdbg-all" clang++ --ld-path="$wild" "$dir"/obj/*.o -o "$dir/cxxdbg.bin"
  ln -sfn "$out/cap-cxxdbg-all/0" "$out/cap-cxxdbg"
}

make_tinyc
make_cxxdbg
echo "captures: $out/cap-tinyc $out/cap-cxxdbg"
