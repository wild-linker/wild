## Time

This page has benchmarks for linking Wasm on a AMD Ryzen 9 9955HX 16-Core (32 thread) processor with
92GiB of RAM.

Wild's Wasm support doesn't yet handle debug info, so all these benchmarks were built without
explicit debug info, although a small amount of debug info might be present from precompiled
libraries.

### ripgrep-wasm - time
![Time to link ripgrep-wasm](images/ryzen-wasm/ripgrep-wasm-time.svg)

### spidermonkey-wasm - time
![Time to link spidermonkey-wasm](images/ryzen-wasm/spidermonkey-wasm-time.svg)

### wild-release-wasm - time
![Time to link wild-release-wasm](images/ryzen-wasm/wild-release-wasm-time.svg)

### cruby-wasm - time
![Time to link cruby-wasm](images/ryzen-wasm/cruby-wasm-time.svg)

### cpython-wasm - time
![Time to link cpython-wasm](images/ryzen-wasm/cpython-wasm-time.svg)

### sqlite-wasm - time
![Time to link sqlite-wasm](images/ryzen-wasm/sqlite-wasm-time.svg)

### spidermonkey-wasm.threads - time
![Time to link spidermonkey-wasm.threads](images/ryzen-wasm/spidermonkey-wasm.threads-time.svg)

## Memory

### ripgrep-wasm - memory
![Memory consumption while linking ripgrep-wasm](images/ryzen-wasm/ripgrep-wasm-memory.svg)

### spidermonkey-wasm - memory
![Memory consumption while linking spidermonkey-wasm](images/ryzen-wasm/spidermonkey-wasm-memory.svg)

### wild-release-wasm - memory
![Memory consumption while linking wild-release-wasm](images/ryzen-wasm/wild-release-wasm-memory.svg)

### cruby-wasm - memory
![Memory consumption while linking cruby-wasm](images/ryzen-wasm/cruby-wasm-memory.svg)

### cpython-wasm - memory
![Memory consumption while linking cpython-wasm](images/ryzen-wasm/cpython-wasm-memory.svg)

### sqlite-wasm - memory
![Memory consumption while linking sqlite-wasm](images/ryzen-wasm/sqlite-wasm-memory.svg)

### spidermonkey-wasm.threads - memory
![Memory consumption while linking spidermonkey-wasm.threads](images/ryzen-wasm/spidermonkey-wasm.threads-memory.svg)

