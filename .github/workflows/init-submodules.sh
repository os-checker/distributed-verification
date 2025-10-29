#!/usr/bin/bash

set -eoux pipefail

need_k=$((50 * 1024 * 1024))
avail_k=$(df -k /mnt | awk 'NR==2{print $4}')

df -alh

if ((avail_k < need_k)); then
  git submodule update --init --recursive kani verify-rust-std
  exit 0
fi

echo "Mount /mnt (/dev/sda1) to verify-rust-std because the available space is $((avail_k / 1024 / 1024))G"

TARGET_DIR=$PWD/verify-rust-std/target

# We can't store data on /dev/sda1 by mounting verify-rust-std directly.
# Kani stores verify-rust-std/target which takes a lot space, so we mount that folder.
# Note: there os a verify-rust-std/library/target folder as well which is not mounted to the sda1 dist.
mkdir "$TARGET_DIR"

sudo mkdir -p /mnt/my
sudo chown runner:runner /mnt/my
sudo chmod 755 /mnt/my

sudo mount --bind /mnt/my "$TARGET_DIR"

git submodule update --init --recursive kani verify-rust-std

ls -alh "$TARGET_DIR"
ls -alh /mnt/my

df -alh
