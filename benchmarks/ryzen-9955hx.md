# Benchmarks

This page contains benchmarks of linking various programs. The benchmarks are just running the
linker, not the compiler. All benchmarks are run on an AMD Ryzen 9 9955HX 16-Core (32 thread)
processor with 92GiB of RAM. The output file is on tmpfs unless otherwise stated.

The benchmarks include several other linkers for Linux as well as several recent versions of Wild.
This allows us to both compare between linkers and to see how Wild's performance is changing over
time.

We only include GNU ld (BFD) in a few of the benchmarks since it's generally a lot slower than the
other linkers and including it messes up the scaling on the charts. It does do OK on memory
consumption though.

The little white ticks on each bar are 99% confidence intervals and show how accurate we think our
mean value is. i.e. if we reran the benchmark 100 times, we'd expect that 99 times we'd get a mean
within that the range of the white tick.

## Execution time

### chrome - time

We start with linking the Chrome web browser (technically Chromium). This is a very large codebase
and a pretty large binary.

![Benchmark of linking chrome](images/ryzen-9955hx/chrome-time.svg)

### chrome-crel - time

CREL relocations are a new format for relocations that is significantly more compact. Wild has
supported these for a couple of releases, however we hadn't benchmarked non-trivial programs with
CREL relocations until just prior to the 0.8.0 release. It turned out that we had a bug that caused
memory usage to blow up, making it impractical to link non-trivial programs. This is now fixed, so
we benchmark CREL relocations from 0.8.0. Chrome uses CREL relocations by default.

![Benchmark of linking chrome-crel](images/ryzen-9955hx/chrome-crel-time.svg)

### chrome-android - time
![Time to link chrome-android](images/ryzen-9955hx/chrome-android-time.svg)

### mold-crel - time

For a smaller CREL benchmark, we link Mold, with the compiler configured to emit CREL relocations
(not the default configuration).

![Benchmark of linking mold-crel](images/ryzen-9955hx/mold-crel-time.svg)

### clang-debug - time

Clang with debug info has massive quantities of debug strings that need to be deduplicated. So link
time here is dominated by how quickly we can deduplicate strings.

![Benchmark of linking clang-debug](images/ryzen-9955hx/clang-debug-time.svg)

### clang-debug-strip - time

This benchmark is the same as the previous one, but with the extra flag `--strip-debug`. It shows to
what extent linkers are able to take advantage of the fact that they don't need to emit the debug
info that's in the input files.

![Time to link clang-debug-strip](images/ryzen-9955hx/clang-debug-strip-time.svg)

### clang-release - time
![Benchmark of linking clang-release](images/ryzen-9955hx/clang-release-time.svg)

### rust-analyzer - time
![Benchmark of linking rust-analyzer](images/ryzen-9955hx/rust-analyzer-time.svg)

### rust-analyzer-riscv - time
![Benchmark of linking rust-analyzer-riscv](images/ryzen-9955hx/rust-analyzer-riscv-time.svg)

### zed-release - time
![Benchmark of linking zed-release](images/ryzen-9955hx/zed-release-time.svg)

### zed-debug - time
![Time to link zed-debug](images/ryzen-9955hx/zed-debug-time.svg)

### bevy-dylib - time
![Benchmark of linking bevy-dylib](images/ryzen-9955hx/bevy-dylib-time.svg)

### librustc-driver - time

Rustc is quite large. Most of the code however goes into librustc-driver. Rustc itself it pretty
quick to link, so we only benchmark linking of librustc-driver.

![Benchmark of linking librustc-driver](images/ryzen-9955hx/librustc-driver-time.svg)

### ripgrep - time

Ripgrep is a popular grep alternative. It's got a pretty lean dependency tree, so is fairly fast to
link.

![Benchmark of linking ripgrep](images/ryzen-9955hx/ripgrep-time.svg)

### wild - time

Wild linking itself.

![Benchmark of linking wild](images/ryzen-9955hx/wild-time.svg)

### wild-riscv - time

Wild linking a risc-v version of itself. We're still doing the linking on x86_64, but the binary
being produced is for risc-v.

![Benchmark of linking wild-riscv](images/ryzen-9955hx/wild-riscv-time.svg)

### wild-aarch64 - time
![Time to link wild-aarch64](images/ryzen-9955hx/wild-aarch64-time.svg)

### wild-loongarch64 - time
![Time to link wild-loongarch64](images/ryzen-9955hx/wild-loongarch64-time.svg)

### wild-debug - time
![Time to link wild-debug](images/ryzen-9955hx/wild-debug-time.svg)

Wild linking itself with linker-plugin LTO. This is much much slower, since the linker plugin ends
up doing heaps of work during linking.

### wild-lto - time
![Time to link wild-lto](images/ryzen-9955hx/wild-lto-time.svg)

### rust-hello-world - time

Now a couple of trivial, hello-world programs. One written in Rust and one written in C.

![Benchmark of linking rust-hello-world](images/ryzen-9955hx/rust-hello-world-time.svg)

### c-hello-world - time
![Benchmark of linking c-hello-world](images/ryzen-9955hx/c-hello-world-time.svg)

The next benchmark shows linking with --gdb-index which builds an index for use by GDB. Note that
Wild didn't support this until 0.10.0.

### mold-gdb-index - time
![Time to link mold-gdb-index](images/ryzen-9955hx/mold-gdb-index-time.svg)

## Filesystem performance

How fast linkers are able to write the output file depends a lot on what filesystem that output file
is on as well as various OS-specific tweaks that the linker may have done which can help (or in some
cases hurt) performance on that particular filesystem. Other factors, like whether the output file
already exists can also have a substantial effect on write performance.

### zed-debug.ext4-no-delete-output - time
![Time to link zed-debug.ext4-no-delete-output](images/ryzen-9955hx/zed-debug.ext4-no-delete-output-time.svg)

### zed-debug.ext4-delete-output - time
![Time to link zed-debug.ext4-delete-output](images/ryzen-9955hx/zed-debug.ext4-delete-output-time.svg)

### zed-debug.xfs-delete-output - time
![Time to link zed-debug.xfs-delete-output](images/ryzen-9955hx/zed-debug.xfs-delete-output-time.svg)

### zed-debug.btrfs-delete-output - time
![Time to link zed-debug.btrfs-delete-output](images/ryzen-9955hx/zed-debug.btrfs-delete-output-time.svg)

### zed-debug.threads-tmpfs - time
![Time to link zed-debug.threads-tmpfs](images/ryzen-9955hx/zed-debug.threads-tmpfs-time.svg)

### zed-debug.threads-ext4-delete-output - time
![Time to link zed-debug.threads-ext4-delete-output](images/ryzen-9955hx/zed-debug.threads-ext4-delete-output-time.svg)

## Memory consumption

We care not just about execution time, but also memory consumption. Here are the same benchmarks as
above repeated, but this time measuring the peak memory consumption of the linkers.

### chrome - memory
![Memory consumption while linking chrome](images/ryzen-9955hx/chrome-memory.svg)

### chrome-crel - memory
![Memory consumption while linking chrome-crel](images/ryzen-9955hx/chrome-crel-memory.svg)

### chrome-android - memory
![Memory consumption while linking chrome-android](images/ryzen-9955hx/chrome-android-memory.svg)

### mold-crel - memory
![Memory consumption while linking mold-crel](images/ryzen-9955hx/mold-crel-memory.svg)

### clang-debug - memory
![Memory consumption while linking clang-debug](images/ryzen-9955hx/clang-debug-memory.svg)

### clang-debug-strip - memory
![Memory consumption while linking clang-debug-strip](images/ryzen-9955hx/clang-debug-strip-memory.svg)

### clang-release - memory
![Memory consumption while linking clang-release](images/ryzen-9955hx/clang-release-memory.svg)

### rust-analyzer - memory
![Memory consumption while linking rust-analyzer](images/ryzen-9955hx/rust-analyzer-memory.svg)

### rust-analyzer-riscv - memory
![Memory consumption while linking rust-analyzer-riscv](images/ryzen-9955hx/rust-analyzer-riscv-memory.svg)

### zed-release - memory
![Memory consumption while linking zed-release](images/ryzen-9955hx/zed-release-memory.svg)

### zed-debug - memory
![Memory consumption while linking zed-debug](images/ryzen-9955hx/zed-debug-memory.svg)

### bevy-dylib - memory
![Memory consumption while linking bevy-dylib](images/ryzen-9955hx/bevy-dylib-memory.svg)

### librustc-driver - memory
![Memory consumption while linking librustc-driver](images/ryzen-9955hx/librustc-driver-memory.svg)

### ripgrep - memory
![Memory consumption while linking ripgrep](images/ryzen-9955hx/ripgrep-memory.svg)

### wild - memory
![Memory consumption while linking wild](images/ryzen-9955hx/wild-memory.svg)

### wild-riscv - memory
![Memory consumption while linking wild-riscv](images/ryzen-9955hx/wild-riscv-memory.svg)

### wild-aarch64 - memory
![Memory consumption while linking wild-aarch64](images/ryzen-9955hx/wild-aarch64-memory.svg)

### wild-loongarch64 - memory
![Memory consumption while linking wild-loongarch64](images/ryzen-9955hx/wild-loongarch64-memory.svg)

### wild-debug - memory
![Memory consumption while linking wild-debug](images/ryzen-9955hx/wild-debug-memory.svg)

### wild-lto - memory
![Memory consumption while linking wild-lto](images/ryzen-9955hx/wild-lto-memory.svg)

### rust-hello-world - memory
![Memory consumption while linking rust-hello-world](images/ryzen-9955hx/rust-hello-world-memory.svg)

### c-hello-world - memory
![Memory consumption while linking c-hello-world](images/ryzen-9955hx/c-hello-world-memory.svg)

### mold-gdb-index - memory
![Memory consumption while linking mold-gdb-index](images/ryzen-9955hx/mold-gdb-index-memory.svg)
## UNGROUPED
### zed-debug.ext4-no-delete-output - memory
![Memory consumption while linking zed-debug.ext4-no-delete-output](images/ryzen-9955hx/zed-debug.ext4-no-delete-output-memory.svg)

### zed-debug.ext4-delete-output - memory
![Memory consumption while linking zed-debug.ext4-delete-output](images/ryzen-9955hx/zed-debug.ext4-delete-output-memory.svg)

### zed-debug.xfs-delete-output - memory
![Memory consumption while linking zed-debug.xfs-delete-output](images/ryzen-9955hx/zed-debug.xfs-delete-output-memory.svg)

### zed-debug.btrfs-delete-output - memory
![Memory consumption while linking zed-debug.btrfs-delete-output](images/ryzen-9955hx/zed-debug.btrfs-delete-output-memory.svg)

### zed-debug.threads-tmpfs - memory
![Memory consumption while linking zed-debug.threads-tmpfs](images/ryzen-9955hx/zed-debug.threads-tmpfs-memory.svg)

### zed-debug.threads-ext4-delete-output - memory
![Memory consumption while linking zed-debug.threads-ext4-delete-output](images/ryzen-9955hx/zed-debug.threads-ext4-delete-output-memory.svg)

