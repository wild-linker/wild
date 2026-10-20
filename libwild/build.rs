fn main() {
    if std::env::var_os("CARGO_FEATURE_LLVM_LTO").is_some() {
        build_llvm();
    }
}

fn build_llvm() {
    const MIN_LLVM_VERSION: u32 = 18;
    const MAX_LLVM_VERSION: u32 = 23;

    println!("cargo:rerun-if-env-changed=LLVM_CONFIG");
    println!("cargo:rerun-if-env-changed=WILD_ALLOW_NEWER_LLVM");
    println!("cargo:rerun-if-changed=src/llvm_lto.cpp");
    println!("cargo:rerun-if-changed=src/llvm_lto.h");

    let llvm_config = std::env::var_os("LLVM_CONFIG").unwrap_or_else(|| "llvm-config".into());

    let query = |args: &[&str]| {
        let output = std::process::Command::new(&llvm_config)
            .args(args)
            .output()
            .unwrap_or_else(|error| {
                panic!(
                    "llvm-lto requires LLVM {MIN_LLVM_VERSION} through {MAX_LLVM_VERSION} development libraries; set LLVM_CONFIG: {error}"
                )
            });

        assert!(
            output.status.success(),
            "llvm-config failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout)
            .expect("Non-UTF8 llvm-config output")
            .trim()
            .to_owned()
    };

    let version = query(&["--version"]);
    let major = version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .unwrap_or_else(|| panic!("Invalid LLVM version: {version}"));

    assert!(
        major >= MIN_LLVM_VERSION,
        "llvm-lto requires LLVM {MIN_LLVM_VERSION} or newer, found {version}"
    );

    if major > MAX_LLVM_VERSION {
        assert!(
            std::env::var("WILD_ALLOW_NEWER_LLVM").as_deref() == Ok("1"),
            "llvm-lto supports LLVM through {MAX_LLVM_VERSION}, found {version}; set WILD_ALLOW_NEWER_LLVM=1 to attempt building with this untested version"
        );

        println!(
            "cargo:warning=Building llvm-lto with untested LLVM {version}; supported versions are {MIN_LLVM_VERSION} through {MAX_LLVM_VERSION}"
        );
    }

    let mut build = cc::Build::new();
    let include_dir = query(&["--includedir"]);

    // Moving /usr/include ahead of C++ headers breaks their #include_next directives.
    if include_dir != "/usr/include" {
        build.flag("-isystem").flag(include_dir);
    }

    build
        .cpp(true)
        .std("c++17")
        .file("src/llvm_lto.cpp")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .warnings(true)
        .compile("wild_llvm_lto");

    println!("cargo:rustc-link-search=native={}", query(&["--libdir"]));

    for lib in query(&["--link-shared", "--libs"]).split_whitespace() {
        let name = lib
            .strip_prefix("-l")
            .expect("Unexpected llvm-config library name");

        println!("cargo:rustc-link-lib=dylib={name}");
    }
}
