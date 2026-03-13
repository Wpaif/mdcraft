#!/usr/bin/env bash
set -euo pipefail

# Build all configured targets for mdcraft.
# Usage:
#   scripts/build-all.sh
#   scripts/build-all.sh --release
#   scripts/build-all.sh --linking dynamic
#   scripts/build-all.sh --linking static --release
#   scripts/build-all.sh --windows-target auto
#   scripts/build-all.sh --windows-target native --release
#   scripts/build-all.sh --windows-target gnu --release
#   scripts/build-all.sh --windows-target msvc --release
#
# Host behavior:
#   - Linux: builds Linux x64/arm64 + Windows target.
#   - Windows: builds only Windows target (native flow).

mode="debug"
release_flag=""
linking="dynamic"
windows_target="auto"

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
    --windows-target)
      windows_target="${2:-}"
      if [[ "$windows_target" != "auto" && "$windows_target" != "native" && "$windows_target" != "gnu" && "$windows_target" != "msvc" ]]; then
        echo "Erro: --windows-target deve ser 'auto', 'native', 'gnu' ou 'msvc'."
        exit 1
      fi
      shift 2
      ;;
    -h|--help)
      sed -n '1,22p' "$0"
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
echo "Windows target option: ${windows_target}"

host_uname="$(uname -s 2>/dev/null || echo unknown)"
host_os="other"
case "$host_uname" in
  Linux*)
    host_os="linux"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    host_os="windows"
    ;;
esac

if [[ "${OS:-}" == "Windows_NT" ]]; then
  host_os="windows"
fi

resolved_windows_target=""
case "$windows_target" in
  auto)
    if [[ "$host_os" == "windows" ]]; then
      resolved_windows_target="x86_64-pc-windows-msvc"
    else
      resolved_windows_target="x86_64-pc-windows-gnu"
    fi
    ;;
  native)
    if [[ "$host_os" == "windows" ]]; then
      resolved_windows_target="x86_64-pc-windows-msvc"
    else
      resolved_windows_target="x86_64-pc-windows-gnu"
      echo "Aviso: --windows-target native em host Linux usa cross para x86_64-pc-windows-gnu."
    fi
    ;;
  gnu)
    resolved_windows_target="x86_64-pc-windows-gnu"
    ;;
  msvc)
    if [[ "$host_os" != "windows" ]]; then
      echo "Erro: target MSVC requer host Windows (ou ambiente MSVC configurado)."
      exit 1
    fi
    resolved_windows_target="x86_64-pc-windows-msvc"
    ;;
esac

echo "Host OS detectado: ${host_os} (${host_uname})"
echo "Resolved Windows target: ${resolved_windows_target}"

if [[ "$linking" == "static" ]]; then
  linux_x64_target="x86_64-unknown-linux-musl"
  linux_arm64_target="aarch64-unknown-linux-musl"
  win_rustflags="-C target-feature=+crt-static"
else
  linux_x64_target="x86_64-unknown-linux-gnu"
  linux_arm64_target="aarch64-unknown-linux-gnu"
  win_rustflags="-C target-feature=-crt-static"
fi

if [[ "$host_os" == "linux" ]]; then
  echo "==> Building Linux x64 (${mode})"
  cargo build ${release_flag} --target "${linux_x64_target}"

  echo "==> Building Linux arm64 (${mode})"
  cargo build ${release_flag} --target "${linux_arm64_target}"
elif [[ "$host_os" == "windows" ]]; then
  echo "==> Host Windows detectado: pulando targets Linux neste script."
else
  echo "Erro: host nao suportado (${host_uname}). Use Linux, MSYS2, Git Bash ou Cygwin no Windows."
  exit 1
fi

echo "==> Building Windows x64 (${mode}) target=${resolved_windows_target}"
if [[ "$resolved_windows_target" == "x86_64-pc-windows-gnu" ]]; then
  CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="${win_rustflags}" \
    cargo build ${release_flag} --target x86_64-pc-windows-gnu
else
  if [[ "$linking" == "static" ]]; then
    echo "Aviso: --linking static/dynamic so afeta o target windows-gnu neste script."
  fi
  cargo build ${release_flag} --target x86_64-pc-windows-msvc
fi

echo "Done."
