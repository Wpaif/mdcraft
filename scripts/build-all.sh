#!/usr/bin/env bash
set -euo pipefail

# Build all configured targets for mdcraft.
# Usage:
#   scripts/build-all.sh
#   scripts/build-all.sh --release
#   scripts/build-all.sh --linking dynamic
#   scripts/build-all.sh --linking static --release

mode="debug"
release_flag=""
linking="dynamic"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release)
      mode="release"
      release_flag="--release"
      shift
      ;;
    --linking)
      linking="${2:-}"
      if [[ "$linking" != "dynamic" && "$linking" != "static" ]]; then
        echo "Erro: --linking deve ser 'dynamic' ou 'static'."
        exit 1
      fi
      shift 2
      ;;
    -h|--help)
      sed -n '1,18p' "$0"
      exit 0
      ;;
    *)
      echo "Argumento inválido: $1"
      exit 1
      ;;
  esac
done

echo "Mode: ${mode}"
echo "Linking: ${linking}"

if [[ "$linking" == "static" ]]; then
  linux_x64_target="x86_64-unknown-linux-musl"
  linux_arm64_target="aarch64-unknown-linux-musl"
  win_rustflags="-C target-feature=+crt-static"
else
  linux_x64_target="x86_64-unknown-linux-gnu"
  linux_arm64_target="aarch64-unknown-linux-gnu"
  win_rustflags="-C target-feature=-crt-static"
fi

echo "==> Building Linux x64 (${mode})"
cargo build ${release_flag} --target "${linux_x64_target}"

echo "==> Building Linux arm64 (${mode})"
cargo build ${release_flag} --target "${linux_arm64_target}"

echo "==> Building Windows x64 (${mode})"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="${win_rustflags}" \
  cargo build ${release_flag} --target x86_64-pc-windows-gnu

echo "Done."
