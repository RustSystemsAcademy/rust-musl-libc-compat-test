mod kernel;
mod runner;
mod security;
mod tests;

/// Build provenance baked in at compile time.
pub struct BuildInfo {
    pub rustc_version:      &'static str,
    pub musl_gcc_version:   &'static str,
    pub musl_libc_version:  &'static str,
    pub build_timestamp:    &'static str,
    pub build_target:       &'static str,
}

pub const BUILD: BuildInfo = BuildInfo {
    rustc_version:     env!("BUILD_RUSTC_VERSION"),
    musl_gcc_version:  env!("BUILD_MUSL_GCC_VERSION"),
    musl_libc_version: env!("BUILD_MUSL_LIBC_VERSION"),
    build_timestamp:   env!("BUILD_TIMESTAMP"),
    build_target:      env!("BUILD_TARGET"),
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        print_version();
        return;
    }

    let kernel = kernel::KernelVersion::current();
    let tests  = tests::all();
    let code   = runner::run_all(&tests, &kernel, &BUILD);
    std::process::exit(code);
}

fn print_version() {
    println!("musl-compat-test {}", env!("CARGO_PKG_VERSION"));
    println!("  rustc          : {}", BUILD.rustc_version);
    println!("  musl-gcc       : {}", BUILD.musl_gcc_version);
    println!("  musl libc      : {}", BUILD.musl_libc_version);
    println!("  build target   : {}", BUILD.build_target);
    println!("  build unix ts  : {}", BUILD.build_timestamp);
}
