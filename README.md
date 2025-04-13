# Distributed and resource-efficient verification for verify-rust-std

Context: [Distributed and resource-efficient verification][distributed], GSoC Rust 2025

[distributed]: https://github.com/rust-lang/google-summer-of-code/tree/45141d74c28d91e114cf621d2d56aea6c3f82547?tab=readme-ov-file#distributed-and-resource-efficient-verification

## Steps

The list in very incomplete at the moment.

- [ ] Analyze kani's proofs and contracts
    - [x] Rust Compiler Driver
    - [x] Reuse reachability code from kani
    - [x] Traverse calls starting from proofs and contracts
- [ ] Generate proofs that need to rerun
    - [ ] Compute difference
    - [ ] Emit JSON
- [ ] CI setup
    - [ ] Run kani
    - [ ] Run ESBMC ??
