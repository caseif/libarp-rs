# libarp

ARP is a binary format for packing resource files in a structured manner. libarp is a reference implementation of
pack/unpack functionality for the format.

This repository contains a Rust rewrite of the original [libarp](https://github.com/caseif/libarp) library which is
written in C. Additionally, a Rust rewrite of [arptool](https://github.com/caseif/arptool) is included in this crate as
well.

ARP's full specification can be found in the [SPEC.md](https://github.com/caseif/libarp/doc/SPEC.md) file in the
original libarp repository.

## Compiling

To compile libarp, simply run `cargo build`. By default, the `arptool` CLI will also be built and can be disabled via
the `arptool` feature flag.

## License

libarp and arptool are made available under the [MIT License](https://opensource.org/licenses/MIT). You may use, modify, and
distribute the project within its terms.

The ARP specification is made available under the
[Apache License, Version 2.0](https://opensource.org/licenses/Apache-2.0). You may use, modify and redistribute the
specification within its terms.
