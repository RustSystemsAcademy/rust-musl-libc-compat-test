use std::process::Command;
use std::env;

fn main() {
    // Rust toolchain version
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    // musl-gcc version — this is the linker wrapper, tells us the system musl
    // Note: the musl *libc* linked in comes from the Rust toolchain, not this,
    // but recording both is useful for diagnosing linker issues
    let musl_gcc = Command::new("musl-gcc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stderr).lines().next()
                     .unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "not found".into());

    // The musl libc version bundled *inside* the Rust toolchain.
    // It lives in the sysroot — find it and extract the version from the header.
    let musl_sysroot_version = musl_version_from_sysroot()
        .unwrap_or_else(|| "unknown".into());

    // Build timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into());

    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={}", rustc);
    println!("cargo:rustc-env=BUILD_MUSL_GCC_VERSION={}", musl_gcc);
    println!("cargo:rustc-env=BUILD_MUSL_LIBC_VERSION={}", musl_sysroot_version);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
    println!("cargo:rustc-env=BUILD_TARGET={}", env::var("TARGET").unwrap_or_default());

    // Re-run only if the toolchain actually changes, not on every build
    println!("cargo:rerun-if-changed=build.rs");
}

/// Locate the musl version header inside the Rust toolchain's bundled sysroot.
/// Rust installs musl headers at:
///   ~/.rustup/toolchains/<toolchain>/lib/rustlib/x86_64-unknown-linux-musl/
///     ... which doesn't have a version file directly, but the musl
///   include path has <features.h> with a __musl__ define (no version there),
///   however the version.h or Makefile equivalent is in the source.
///
/// The most reliable approach: ask rustc for the sysroot, then check the
/// musl include path for version.h.
fn musl_version_from_sysroot() -> Option<String> {
    let sysroot = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    // musl headers are at <sysroot>/lib/rustlib/x86_64-unknown-linux-musl/include
    // but version info is harder to extract from headers alone.
    // The definitive source: look for the musl version in the bundled
    // libc.rlib debug info or the musl source version string.
    //
    // Practical fallback: extract from the libc crate's known mapping,
    // or just record the Rust toolchain version and let that imply musl version.
    let version_paths = [
        format!("{}/lib/rustlib/x86_64-unknown-linux-musl/lib/self-contained/libc.a", sysroot),
    ];

    // If the libc.a exists, we know the musl sysroot is present
    for p in &version_paths {
        if std::path::Path::new(p).exists() {
            // Can't easily extract version from .a without objdump/nm,
            // so record the sysroot path — correlate manually with toolchain release notes
            return Some(format!("bundled in Rust sysroot ({})", sysroot));
        }
    }

    None
}
