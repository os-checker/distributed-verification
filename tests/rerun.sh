# Remove old snapshots
rm tests/snapshots -r

# Create snapshots folders because expect_file won't create parent folders.
mkdir tests/snapshots
mkdir tests/snapshots/stat
mkdir tests/snapshots/simplified
mkdir tests/snapshots/kani_list
mkdir tests/snapshots/compare
mkdir tests/snapshots/proofs
mkdir tests/snapshots/proofs/by_macros
mkdir tests/snapshots/verify-rust-std

# UPDATE_EXPECT=1
cargo test -- --nocapture
