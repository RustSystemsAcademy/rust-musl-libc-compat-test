# Roadmap

This file captures the next test additions that are worth programming into the suite, along with the specific Linux functionality each one is intended to validate.

## Recommended Additions

### `proc: fork/exec self`

Why add it:
The current suite checks `/proc/self/exe` readability, but it does not prove that `execve` works correctly for this static musl binary. Spawning the current binary with `--version` or a small `--self-check` mode would exercise process startup more directly.

Linux functionality actually being tested:
- `fork`/`clone` + `execve` process creation path
- ELF binary startup for the produced static musl executable
- argv handling across exec
- environment inheritance across exec
- stdout/stderr pipe handling through `std::process::Command`
- `/proc/self/exe` usability as an execution target, if that route is used

Why it matters for this project:
This is closer to the real success condition than a pure metadata check. A compatibility binary that starts once but cannot be re-executed cleanly on the target host is still suspect.

### `io: pipe read/write`

Why add it:
The suite currently exercises filesystem I/O and TCP sockets, but not anonymous file descriptors. A simple pipe round-trip is cheap and hits a different class of libc and kernel interactions.

Linux functionality actually being tested:
- `pipe` or `pipe2`
- blocking file descriptor reads and writes
- anonymous kernel-managed byte streams
- close semantics and EOF behavior
- libc wrappers around unbuffered descriptor I/O

Why it matters for this project:
It gives coverage to core Unix I/O primitives that are independent of both the filesystem and the network stack.

### `net: UDP loopback`

Why add it:
The current network coverage only exercises TCP loopback. UDP uses a different socket behavior model and is worth validating separately.

Linux functionality actually being tested:
- `socket` creation for datagram sockets
- `bind`, `sendto`, `recvfrom` or connected UDP send/recv
- loopback IPv4 routing
- datagram delivery semantics without stream state

Why it matters for this project:
It broadens network coverage without introducing outside dependencies or requiring privileged ports.

### `fs: rename + unlink while open`

Why add it:
The suite covers basic create/read/write, symlinks, and directory traversal, but not common Unix pathname semantics around open handles, renames, and unlinks.

Linux functionality actually being tested:
- `open`
- `rename`
- `unlink`
- continued file access through an already-open descriptor after pathname changes
- VFS behavior for inode-backed files independent of directory entry lifetime

Why it matters for this project:
This is a stronger Unix compatibility signal than another basic file read. It validates behavior many real tools rely on for temp files, atomic updates, and log rotation patterns.

### `net: DNS/localhost resolution`

Why add it:
Name resolution is one of the more common musl-versus-host-environment friction points. A `ToSocketAddrs("localhost:0")` or `getaddrinfo("localhost")` test would catch bad `/etc/hosts` or resolver assumptions.

Linux functionality actually being tested:
- libc name resolution path via `getaddrinfo`
- `/etc/hosts` handling for `localhost`
- resolver configuration parsing where relevant
- address family and socket address construction

Why it matters for this project:
This checks a real-world integration edge that often fails differently from raw socket creation.

Operational note:
This one is more environment-sensitive than the other recommendations, so it should likely be optional or skippable rather than treated as a hard failure everywhere.

### `thread: condvar or channel wakeup`

Why add it:
The suite currently covers thread spawn/join, TLS isolation, and barriers, but not blocking wake/sleep coordination. A `Condvar` test would exercise futex-backed synchronization more directly.

Linux functionality actually being tested:
- pthread-backed thread synchronization
- futex-based wait/wake behavior underneath Rust synchronization primitives
- scheduler wakeups and handoff between blocked threads
- mutex + condition variable coordination correctness

Why it matters for this project:
It expands concurrency coverage from “threads exist” to “threads can block and be woken correctly,” which is a more meaningful runtime check.

### `fs: unix domain socket pair`

Why add it:
The suite currently covers TCP loopback and regular filesystem operations, but not local IPC through Unix sockets. This is a common Linux primitive used by many real-world tools and exercises a different path than TCP.

Linux functionality actually being tested:
- `socketpair` or `AF_UNIX` socket creation
- local bidirectional stream semantics without the IP stack
- file descriptor passing readiness, if extended later
- libc wrappers around Unix-domain socket I/O

Why it matters for this project:
It broadens local IPC coverage and helps distinguish “network stack works” from “general socket and descriptor behavior works.”

### `proc: signal handling round-trip`

Why add it:
The current suite does not exercise signal delivery at all. A small self-signal test using `SIGUSR1` would validate a core Unix runtime behavior without requiring privileges.

Linux functionality actually being tested:
- `sigaction`
- signal mask and handler installation
- signal delivery to the current process or thread
- libc signal trampoline behavior

Why it matters for this project:
Signals are foundational Unix behavior and are often involved in supervisor shutdown, timeouts, and diagnostics. A static binary that mishandles them is not fully trustworthy.

### `fs: metadata and permissions sanity`

Why add it:
The suite verifies file contents, but not ownership and mode bit handling. A basic chmod/stat round-trip would validate another layer of libc and VFS behavior.

Linux functionality actually being tested:
- `stat`/`fstat`
- `chmod`
- Unix permission bits and metadata reporting
- libc path-to-metadata translation

Why it matters for this project:
This is a cheap way to validate file metadata correctness, which matters for packaging, temp files, and execution behavior on target hosts.

## Fixes To Make First

### Fix the incorrect monotonic clock test

Problem:
The current `monotonic_clock` test uses `SystemTime`, which is wall clock time, not a monotonic clock source.

What to change:
- Replace it with a real `CLOCK_MONOTONIC` test via `libc::clock_gettime`, or equivalent direct monotonic timing logic
- Rename the current test if it remains and is intended to validate wall clock progression instead

Linux functionality actually being tested after the fix:
- `clock_gettime(CLOCK_MONOTONIC)`
- monotonic time progression unaffected by wall clock adjustments
- libc interaction with kernel timekeeping APIs

Why it matters for this project:
Right now the test name overstates what is being validated. Fixing it keeps the suite technically honest and avoids reporting false coverage.

### Fix temporary path collisions and cleanup robustness

Problem:
Several tests use stable names under `/tmp`, such as `musl_compat_rw` and `musl_renameat2`. That makes reruns and concurrent runs vulnerable to collisions or leftover files after a crash.

What to change:
- Generate unique per-run temp directories using PID, timestamp, randomness, or `mkdtemp`-style logic
- Centralize temp resource creation and cleanup
- Make cleanup best-effort on failure paths so one broken run does not poison the next

Linux functionality actually being tested after the fix:
- The same filesystem behaviors as today, but without false failures caused by reused pathnames
- Crash-safe cleanup expectations for temp resources

Why it matters for this project:
This improves determinism. A compatibility suite should fail because the host is incompatible, not because a previous run left stale files behind.

### Improve syscall error reporting to include `errno`

Problem:
The direct syscall tests currently report raw negative returns such as `renameat2 returned -1`, which is not enough to diagnose whether the failure was expected policy denial, missing kernel support, or a genuine bug.

What to change:
- Capture and report `std::io::Error::last_os_error()` after failing syscalls
- Include the symbolic context in error messages where practical, for example `EPERM`, `ENOSYS`, or `EINVAL`

Linux functionality actually being tested after the fix:
- The same syscall paths as today, but with errors mapped to real Linux failure modes

Why it matters for this project:
This turns the suite from “pass/fail” into a usable diagnostics tool when something breaks on an older or more restricted host.

## Broader Improvements

### Add machine-readable output mode

What to add:
- A `--json` flag or similar output mode
- Structured per-test fields for `name`, `status`, `reason`, `kernel_gate`, and `error`
- Structured build and host metadata in the header section

Why it matters:
Right now the output is human-readable only. JSON would make it much easier to archive results from multiple hosts and compare behavior across kernels or distributions.

### Add per-test timeouts or hang protection

What to add:
- A small timeout mechanism around tests that could block on I/O or synchronization
- A clear `TIMEOUT` or `FAIL` mode distinct from ordinary assertion failures

Why it matters:
Compatibility probes should fail fast. If a target host wedges on one primitive, the suite should still produce actionable output instead of hanging indefinitely.

Linux functionality actually being protected:
- Blocking socket operations
- synchronization waits
- child process execution paths

### Add a focused self-check CLI mode

What to add:
- A hidden or documented `--self-check` mode intended only for the suite's own `fork/exec self` test
- Minimal output and deterministic exit codes

Why it matters:
This makes the process execution test cleaner than reusing the human-facing main output path, and it provides a stable target for future smoke tests.

### Expand unit tests for non-runtime parsing and detection logic

What to add:
- More unit tests for kernel version parsing in `kernel.rs`
- Unit tests for security-context parsing in `security.rs`
- Unit tests for test registry invariants, such as all names being unique and kernel-gated tests having reasons

Why it matters:
Not every regression requires a target Linux host to catch. The parsing and wiring code should be protected by fast local tests.

### Add CI checks for static-linkage invariants

What to add:
- A CI job that builds the musl target and verifies `ldd`, `file`, and `readelf -d` expectations
- A check that the binary still reports the expected build metadata fields

Why it matters:
This project’s central claim is “static musl binary with predictable behavior.” Static-linkage regressions should be caught automatically before anyone copies an artifact to a test host.

### Improve documentation around expected skip behavior

What to add:
- A matrix showing which tests are expected to `PASS`, `SKIP`, or potentially `SKIP (security)` on older kernels and under confined MAC contexts
- A short section distinguishing unsupported-kernel skips from true failures

Why it matters:
The suite already has gating logic, but the operator guidance still lives mostly in prose. A compact matrix would make interpreting results faster and less error-prone.
