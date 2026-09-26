# Using Wild

Wild is a drop-in replacement for the system linker. GCC or Clang invokes it.

## ELF (e.g. Linux)

The same integration options apply across build systems:

* Clang's exclusive option `--ld-path=wild`
* GCC 16.1+ and Clang's option `-fuse-ld=wild` (note that Clang requires an `ld.wild`
  binary or symlink)
* Generally supported `-B <path>`, where `<path>` is the directory containing an `ld` that points to
  `wild`

### Rust (Cargo)

You can use one of the options mentioned above in `~/.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-Clink-arg=--ld-path=wild"]
```

Or:

```toml
[target.x86_64-unknown-linux-gnu]
# linker = "clang" # Uncomment this line if your GCC is older than version 16.
rustflags = ["-Clink-arg=-fuse-ld=wild"]
```

The following `cargo test` command-line builds and tests a crate using Wild. This has been run
successfully on a few popular crates (e.g. ripgrep, serde, tokio, rand, bitflags). It assumes that
the `wild` binary is on your path. It also depends on the Clang compiler being installed, since GCC
doesn't allow using an arbitrary linker.

```sh
RUSTFLAGS="-Clinker=clang -Clink-args=--ld-path=wild" cargo test
```

Alternatively, with an `ld.wild` symlink pointing at `wild`:

```sh
RUSTFLAGS="-Clinker=clang -Clink-args=-fuse-ld=wild" cargo test
```

#### Illumos specific Cargo configuration

```toml
[target.x86_64-unknown-illumos]
# Absolute path to clang - on OmniOS this is likely something like /opt/ooce/bin/clang.
linker = "/usr/bin/clang"

rustflags = [
    # Will silently delegate to GNU ld or Sun ld unless the absolute path to Wild is provided.
    "-Clink-arg=-fuse-ld=/absolute/path/to/wild"
]
```

### CMake

CMake 4.4 or later supports Wild directly when used with Clang or GCC 16 or later. You can select
Wild as the linker by adding `-DCMAKE_LINKER_TYPE=WILD` to the cmake command-line.

For older versions of cmake, see the generic instructions below.

### C/C++ (autotools, meson, old CMake etc.)

Usually setting `LDFLAGS` is enough, but there are projects that implement their own solutions:

```sh
export LDFLAGS="${LDFLAGS} -fuse-ld=wild"
```

Or (especially useful for older GCC versions), create a symlink `ld` pointing to `wild` and pass the
directory to GCC:

```sh
ln -s /usr/bin/wild /tmp/ld

export CFLAGS="${CFLAGS} -B/tmp"
export CXXFLAGS="${CXXFLAGS} -B/tmp"
export LDFLAGS="${LDFLAGS} -B/tmp"
```

Then configure the project (you might need to remove the configuration cache first) and run your
usual build steps.

Due to the complexity of these build systems, you might want to verify that Wild was used to link a
binary with [readelf](FAQ.md#how-can-i-verify-that-wild-was-used-to-link-a-binary).

## WebAssembly

Wasm linking is experimental. Install Wild with the `wasm` feature:

```sh
cargo install --locked --features wasm wild-linker
```

### Rust (Cargo)

Pass the `wild` binary and `-C linker-flavor=wasm-ld`. The same flags apply to `wasm32-unknown-unknown`.

```sh
RUSTFLAGS="-C linker=path/to/wild -C linker-flavor=wasm-ld" \
  cargo build --target wasm32-wasip1
```

```toml
[target.wasm32-wasip1]
rustflags = ["-C", "linker=path/to/wild", "-C", "linker-flavor=wasm-ld"]
```

### C and C++ (Clang)

As with ELF, point Clang at a directory that contains a `wasm-ld` symlink to `wild`. `-fuse-ld=lld` makes Clang look up that `wasm-ld`. `clang++` takes the same flags.

```sh
mkdir -p /tmp/wild
ln -sf "$(command -v wild)" /tmp/wild/wasm-ld

clang --target=wasm32-wasi -B/tmp/wild -fuse-ld=lld hello.c -o hello.wasm
```

For autotools, Meson, and similar drivers:

```sh
export CC="clang --target=wasm32-wasi"
export CXX="clang++ --target=wasm32-wasi"
export LDFLAGS="${LDFLAGS} -B/tmp/wild -fuse-ld=lld"
```

### CMake

Pass a directory that contains a `wasm-ld` symlink to `wild` and `-fuse-ld=lld`. For C++, set `CMAKE_CXX_COMPILER=clang++` and `CMAKE_CXX_COMPILER_TARGET=wasm32-wasi` as well.

```sh
cmake -S . -B build \
  -DCMAKE_C_COMPILER=clang \
  -DCMAKE_C_COMPILER_TARGET=wasm32-wasi \
  -DCMAKE_EXE_LINKER_FLAGS="-B/tmp/wild -fuse-ld=lld" \
  -DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY
```

## CI

If you'd like to use Wild as your linker for Rust code in CI, see
[wild-action](https://github.com/wild-linker/action).
