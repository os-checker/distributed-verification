#!/usr/bin/bash

set -eoux pipefail

need_k=$((50 * 1024 * 1024))
avail_k=$(df -k /mnt | awk 'NR==2{print $4}')

df -alh

git submodule update --init --recursive kani verify-rust-std

if ((avail_k < need_k)); then
  exit 0
fi

echo "Mount /mnt (/dev/sda1) to verify-rust-std because the available space is $((avail_k / 1024 / 1024))G"

declare -A map_mnt

map_mnt["$PWD/verify-rust-std/target"]="/mnt/verify-rust-std/target"
map_mnt["$PWD/verify-rust-std/library/target"]="/mnt/verify-rust-std/library/target"
map_mnt["$PWD/kani/target"]="/mnt/kani/target"

# We can't store data on /dev/sda1 by mounting verify-rust-std directly.
# Kani stores verify-rust-std/target which takes a lot space, so we mount that folder.
# Note: there os a verify-rust-std/library/target folder as well which is not mounted to the sda1 dist.

for key in "${!map_mnt[@]}"; do
  val=${map_mnt[$key]}
  mkdir -p "$key"
  sudo mkdir -p "$val"
  sudo chown runner:runner "$val"
  sudo chmod 755 "$val"
  sudo mount --bind "$val" "$key"
done

df -alh
