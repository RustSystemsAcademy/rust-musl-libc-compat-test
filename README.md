# Rust Musl Libc Compat Test
Build a simple musl libc compatibility test to run on different versions of Linux.

## Build & Verify
Setting the default target in the `.cargo/config.toml` file means you just run `cargo build --release` without having to remember the `--target` flag every time.  

```shell
# Build
cargo build --release

# Confirm fully static
ldd    target/x86_64-unknown-linux-musl/release/musl-compat-test  # "not a dynamic executable"
file   target/x86_64-unknown-linux-musl/release/musl-compat-test  # "statically linked"
readelf -d target/x86_64-unknown-linux-musl/release/musl-compat-test | grep NEEDED  # no output

# Copy to SUSE 12
scp target/x86_64-unknown-linux-musl/release/musl-compat-test user@suse12host:/tmp/

# Run there
./musl-compat-test

```

## Architecture  
The separation of concerns is: `kernel.rs` knows nothing about tests, `runner.rs` knows nothing about what the tests do, each test module is completely self-contained, and `tests/mod.rs` is the only place that wires the requirements (kernel gates) to the implementations. Adding a new test means adding a function in the appropriate module and one line in `tests/mod.rs`.  

## Root Requirements

None of the tests require root. Every test operates within normal user permissions — ephemeral ports above 1024, `/tmp` for filesystem work, loopback networking, and reading `/proc/self`. You can run the whole suite as any unprivileged user.  

## SELinux and AppArmor Concerns

**SELinux**
The most likely practical issue isn't a test being blocked mid-run — it's the binary failing to execute at all on the test host. On a host with SELinux enforcing, files in `/tmp` have type `tmp_t` and many policies deny executing `tmp_t` files. If you scp the binary to `/tmp/` on a host system and try to run it there, you may hit this.  
  
The fix is simple: copy to `~/` or `/usr/local/bin` instead of `/tmp`, or relabel with `chcon -t bin_t ./musl-compat-test`.  

**AppArmor**
AppArmor works on named profiles attached to specific executables. An unknown binary with no profile runs fully unconfined — so copying a new binary to host with AppArmor and running it is almost certainly unconfined and unrestricted. The concern would only arise if the binary were placed in a path that has an existing AppArmor wildcard profile, which is unusual. The one thing worth detecting is whether AppArmor is even active, and if so, whether our process has a profile or is unconfined.  
  
**Specific tests that are sensitive to security policy:**
`memfd_create` — newer SELinux policies (kernel 4.14+) added explicit `memfd_create` permissions. On an older SELinux policy with a confined domain this can be denied.  
  
`renameat2` with `RENAME_EXCHANGE` — generally fine, but some strict AppArmor profiles block unusual rename flags.  
  
Everything else (filesystem in `/tmp`, loopback TCP, `/proc/self`, threading) is routinely permitted under both frameworks for unconfined processes.  

## What the Output Looks Like

**On Rocky Linux, SELinux enforcing, unconfined user, binary in `~/`:**
```
=== musl ABI compatibility test ===
kernel   : 5.14.0
arch     : x86_64
security : SELinux enforcing | context: unconfined_u:unconfined_r:unconfined_t:s0-s0:c0.c1023
uid      : 1001

  PASS  env: set/get round-trip
  ...
  PASS  syscall: memfd_create(2)
  PASS  syscall: renameat2(2)

results: 15 passed, 0 skipped, 0 failed
```

**On Rocky Linux, binary in `/tmp/` (exec-from-tmp warning):**
```
  WARNING: binary is running from /tmp/musl-compat-test — SELinux may label
  this tmp_t which can prevent execution or restrict capabilities.
  Consider copying to ~/musl-compat-test or /usr/local/bin/ instead.
```

**On SUSE 12, AppArmor active, unconfined:**
```
=== musl ABI compatibility test ===
kernel   : 3.12.49
arch     : x86_64
security : AppArmor active | profile: unconfined
uid      : 1001

  PASS  env: set/get round-trip
  ...
  SKIP  syscall: getrandom(2)          (needs 3.17: SYS_getrandom added in 3.17)
  SKIP  syscall: memfd_create(2)       (needs 3.17: SYS_memfd_create added in 3.17)
  SKIP  syscall: renameat2(2)          (needs 3.15: SYS_renameat2 added in 3.15)

results: 12 passed, 3 skipped, 0 failed
```
