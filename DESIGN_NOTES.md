# Design Notes

## What This Project Does Not Try To Validate

This project is a focused compatibility probe for a static musl binary on different Linux kernels and host security contexts. It is intentionally narrow.

It does not try to validate:

- full application correctness
- every libc surface area
- glibc/NSS compatibility behavior
- plugin or `dlopen` ecosystems
- locale completeness or glibc locale behavior
- CA bundle availability or full TLS trust-chain correctness
- DNS behavior beyond what the current tests explicitly cover
- every filesystem type or mount layout
- every container runtime or seccomp profile
- every CPU microarchitecture variant
- performance characteristics
- memory-usage characteristics
- kernel bugs outside the tested primitives

## Practical Interpretation

A clean run means:

- this specific static musl test binary starts
- the tested Linux primitives behave as expected on that host
- kernel gating and security gating are working as designed

A clean run does not mean:

- any musl-linked application will run everywhere
- your application has no runtime file dependencies
- your application is safe from older-kernel syscall issues
- your application will behave the same as a glibc build

## Why The Scope Is Narrow

The point of this project is to produce a small, interpretable signal:

- is the host broadly compatible with this style of static musl binary
- are expected old-kernel skips happening correctly
- are obvious security-context restrictions visible

Once the scope gets much wider, failures become harder to interpret. This project is more useful as a narrow compatibility probe than as a pretend end-to-end certification suite.
