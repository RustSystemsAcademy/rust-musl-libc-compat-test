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
