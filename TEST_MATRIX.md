# Test Matrix

This matrix describes the expected result shape for the current test suite under the main host categories this project cares about.

Use it to distinguish:

- expected `PASS`
- expected `SKIP` due to kernel age
- expected `SKIP` due to security confinement
- unexpected `FAIL`

If a host produces something outside these expectations, treat it as a compatibility or environment investigation signal.

## Baseline Assumptions

- The binary is built as a static musl executable.
- The binary is run as an unprivileged user.
- The binary is launched from an executable location.
- The host has working loopback networking and a normal `/tmp`.
- The host is not missing standard runtime files needed by the process.

## Legend

- `PASS`: expected to run successfully
- `SKIP (kernel)`: expected to skip because the running kernel is too old
- `SKIP (security)`: expected to skip because the current security context may deny the operation
- `WARN`: expected warning, but not a test failure by itself
- `FAIL`: not expected; investigate

## Current Tests

| Test | Kernel < 3.15 | Kernel 3.15-3.16 | Kernel >= 3.17 | Confined SELinux/AppArmor |
| --- | --- | --- | --- | --- |
| `env: set/get round-trip` | PASS | PASS | PASS | PASS |
| `env: iterate environment` | PASS | PASS | PASS | PASS |
| `fs: create / write / read` | PASS | PASS | PASS | PASS |
| `fs: symlink creation` | PASS | PASS | PASS | PASS |
| `fs: directory traversal` | PASS | PASS | PASS | PASS |
| `thread: spawn + join` | PASS | PASS | PASS | PASS |
| `thread: TLS isolation` | PASS | PASS | PASS | PASS |
| `thread: barrier sync` | PASS | PASS | PASS | PASS |
| `net: TCP loopback` | PASS | PASS | PASS | PASS |
| `time: monotonic clock` | PASS | PASS | PASS | PASS |
| `time: CLOCK_BOOTTIME` | PASS | PASS | PASS | PASS |
| `proc: PID / /proc/self/exe` | PASS | PASS | PASS | PASS |
| `syscall: getrandom(2)` | SKIP (kernel) | SKIP (kernel) | PASS | PASS |
| `syscall: memfd_create(2)` | SKIP (kernel) | SKIP (kernel) | PASS | SKIP (security) |
| `syscall: renameat2(2)` | SKIP (kernel) | PASS | PASS | SKIP (security) |

## Expected Result Profiles

### Old host, no confinement, example: SUSE 12 class kernel (`3.12.x`)

- `12 PASS`
- `3 SKIP (kernel)`
- `0 FAIL`

Expected skipped tests:

- `syscall: getrandom(2)`
- `syscall: memfd_create(2)`
- `syscall: renameat2(2)`

### Mid-era host, no confinement, kernel `3.15.x` or `3.16.x`

- `13 PASS`
- `2 SKIP (kernel)`
- `0 FAIL`

Expected skipped tests:

- `syscall: getrandom(2)`
- `syscall: memfd_create(2)`

Expected newly running test:

- `syscall: renameat2(2)`

### Modern host, no confinement, kernel `>= 3.17`

- `15 PASS`
- `0 SKIP`
- `0 FAIL`

### Modern host, confined SELinux or confined AppArmor

Expected behavior if the process is actually confined:

- `syscall: memfd_create(2)` -> `SKIP (security)`
- `syscall: renameat2(2)` -> `SKIP (security)`

Everything else should still normally `PASS`.

Notes:

- `syscall: getrandom(2)` is not currently security-gated by the runner and is expected to run when the kernel is new enough.
- SELinux enforcing with an `unconfined_t` domain is not treated as confinement in the current implementation.
- AppArmor with profile `unconfined` is not treated as confinement in the current implementation.

## Execution-Location Expectations

These are not test results, but they affect interpretation:

- Running from `/tmp` on an SELinux host may produce a `WARN` about `tmp_t` execution restrictions.
- That warning is expected and should not be interpreted as a failed test.
- A host that refuses to execute the binary from `/tmp` is not a surprise condition; retry from `~/` or another executable location.

## Unexpected Outcomes

Treat these as investigation cases:

- Any `FAIL` on tests that should `PASS` for the current kernel and security context
- Any missing `SKIP (kernel)` on kernels older than the documented syscall minimum
- Any failure to start the binary at all
- Any DNS, certificate, mount, or runtime-file issue that prevents the suite from reaching test execution

## Practical Reading Rule

Success does not always mean `15 PASS`.

Success means:

- all always-safe tests `PASS`
- kernel-gated tests `SKIP` on too-old kernels instead of failing
- security-sensitive tests `SKIP (security)` when confinement is expected
- no unexpected `FAIL` occurs
