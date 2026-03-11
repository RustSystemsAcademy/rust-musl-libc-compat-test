  kernel and host filesystem/security environment.

  The issues you usually hit are:

  - Newer syscalls on older kernels. If your binary or a dependency uses getrandom, memfd_create, renameat2, statx, pidfd_*, copy_file_range, io_uring, or similar, an older kernel may return ENOSYS or behave differently.
  - Different syscall semantics across kernel versions. Sometimes the syscall exists but flags or edge-case behavior differ, so code that works on a modern kernel can fail or misbehave on an older one.
  - DNS and name resolution surprises. musl’s resolver behavior differs from glibc/NSS expectations. If your app assumes glibc-style NSS modules, LDAP-backed users/groups, mDNS plugins, or certain resolver quirks, it may
    not behave the same.
  - Host config dependencies. Even a static binary still depends on runtime files like /etc/resolv.conf, /etc/hosts, /etc/passwd, CA bundles, timezone data, locale data, and /proc or /sys if the app uses them.
  - TLS/certificate paths. HTTPS clients often fail on some hosts because the CA bundle path differs or the host lacks expected cert files.
  - Locale and i18n differences. musl’s locale support is simpler than glibc’s. Programs expecting rich locale behavior, collation, or glibc locale databases can see regressions.
  - Threads, signals, and low-level runtime assumptions. Most code is fine, but software with tight coupling to pthread internals, signal stacks, or glibc-specific behavior can break.
  - dlopen/plugin expectations. Fully static musl binaries do not mix well with ecosystems that expect dynamic plugins or glibc-linked shared objects.
  - Security policy differences. SELinux, AppArmor, seccomp, container runtimes, and mount options can block syscalls, execution from /tmp, memfd_create, unusual renames, or other operations even when the kernel is new
    enough.
  - Filesystem and mount differences. noexec, nodev, nosuid, overlayfs quirks, older procfs/sysfs behavior, and network filesystem differences can all affect runtime behavior.
  - CPU/ABI assumptions. musl does not protect you from building for an instruction set not available on the target CPU.
  - Tooling assumptions. Some third-party crates or native dependencies quietly assume glibc behavior and will compile but fail subtly under musl.

  The practical rule is:

  - musl helps with libc portability across distros
  - it does not guarantee portability across kernel versions or host environments

  For a musl-targeted app, the safest approach is:

  1. Minimize direct dependence on new syscalls.
  2. Gracefully handle ENOSYS, EPERM, and EINVAL.
  3. Test on your oldest supported kernel, not just your build host.
  4. Test under different security contexts and mount layouts.
  5. Be explicit about runtime file dependencies like certs, tzdata, and resolver config.

  If you want, I can turn this into a host-compatibility checklist tailored to your project.
  