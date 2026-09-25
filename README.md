# Wild linker

![Wild logo - drawing of rusty chain links with vines](/images/wild.png)

Wild is a linker with the goal of being very fast for iterative development.

The plan is to eventually make it incremental, however that isn't yet implemented. It is however
already pretty fast even without incremental linking.

### Documentation quick links

* [Usage](USAGE.md)
* [Frequently asked questions](FAQ.md)
* [Linker script support](LINKER_SCRIPT_SUPPORT.md)

## Benchmarks

The goal of Wild is to eventually be very fast via incremental linking. However, we also want to be
as fast as we can be for non-incremental linking and for the initial link when incremental linking is enabled. See [BENCHMARKING.md](BENCHMARKING.md) for details on running benchmarks.

We run benchmarks on a few different systems:

* [Ryzen 9 9955HX (16 core, 32 thread)](benchmarks/ryzen-9955hx.md)
* [2020 era Intel-based laptop with 4 cores and 8 threads](benchmarks/lemp9.md)
* [Raspberry Pi 5](benchmarks/raspberrypi.md)

For example, linking Chromium with CREL relocations on the Ryzen system:

![Benchmark of linking chrome-crel](benchmarks/images/ryzen-9955hx/chrome-crel-time.svg)

## Feature support

The following platforms / architectures are currently supported:

* x86-64 on Linux
* AArch64 (ARM64) on Linux
* RISC-V (riscv64gc) on Linux
* LoongArch64 on Linux
* PPC64LE on Linux (initial support)

The following features are supported:

* Output to statically linked, non-relocatable binaries
* Output to statically linked, position-independent binaries (static-PIE)
* Output to dynamically linked binaries
* Output to shared objects (.so files)
* Rust proc-macros, when linked with Wild work
* Most of the top downloaded crates on crates.io have been tested with Wild and pass their tests
* Debug info (DWARF)
* GNU jobserver support
* Linker script support. See the [linker script support matrix](LINKER_SCRIPT_SUPPORT.md) for details.
* Linker plugin LTO

Here are some of the larger things that aren't yet done:

* Incremental linking
* Mach-O support
* WebAssembly support
* Windows support

## Installation

### From GitHub releases

Download a tarball from the [releases page](https://github.com/wild-linker/wild/releases). Unpack
it and copy the `wild` binary somewhere on your path.

### Cargo binstall

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), you can install wild as
follows:

```sh
cargo binstall wild-linker
```

### Brew

```sh
brew install wild-linker/wild/wild
```

### Build latest release from crates.io

```sh
cargo install --locked wild-linker
```

### Build from git head

To build and install the latest, unreleased code:

```sh
cargo install --locked --bin wild --git https://github.com/wild-linker/wild.git wild-linker
```

### Nix

To use a stable Wild from Nixpkgs:

```nix
let
 wildStdenv = pkgs.useWildLinker pkgs.stdenv;
in
pkgs.callPackage ./package { stdenv = wildStdenv; }
```

to use the latest unstable git revision of wild, see [the nix documentation](./nix/nix.md).

## Contributing

For more information on contributing to `wild` see [CONTRIBUTING.md](CONTRIBUTING.md).

## Chat server

We have a Zulip server for Wild-related chat. You can join
[here](https://wild.zulipchat.com/join/bbopdeg6howwjpaiyowngyde/).

## Further reading

Many of the posts on [David's blog](https://davidlattimore.github.io/) are about various aspects of
the Wild linker.

## Sponsorship

If you'd like to [sponsor this work](https://github.com/sponsors/davidlattimore), that would be very
much appreciated. The more sponsorship I get the longer I can continue to work on this project full
time.

## Code of Conduct

The Wild project adheres to the [Rust code of
conduct](https://rust-lang.org/policies/code-of-conduct/). If you have any moderation concerns or
queries, please email wild-mod@googlegroups.com.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Wild by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
