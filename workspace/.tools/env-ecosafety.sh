#!/usr/bin/env bash
set -euo pipefail

if [ ! -d "/mnt/oss/rustup" ] || [ ! -d "/mnt/oss/cargo" ]; then
  echo "Rust toolchain not found under /mnt/oss; run the mntoss rust setup script." >&2
  exit 1
fi

export RUSTUP_HOME="/mnt/oss/rustup"
export CARGO_HOME="/mnt/oss/cargo"
export PATH="${CARGO_HOME}/bin:${PATH}"

cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec "$@"
