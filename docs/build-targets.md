# Build Targets (x64 / arm64)

Este guia cobre compilacao multi-target do `mdcraft` em host Linux e host Windows.

## 1) Targets suportados

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`
- `x86_64-pc-windows-gnu`
- `x86_64-pc-windows-msvc` (compilacao nativa no Windows)

## 2) Instalar targets do Rust

```bash
rustup target add \
  x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
  x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
  x86_64-pc-windows-gnu x86_64-pc-windows-msvc
```

## 3) Toolchains por ambiente

### Host Linux (cross)

```bash
sudo apt-get update
sudo apt-get install -y \
  gcc-aarch64-linux-gnu \
  binutils-aarch64-linux-gnu \
  gcc-mingw-w64-x86-64 \
  musl-tools
```

Para `aarch64-unknown-linux-musl`, o pacote pode variar por distribuicao:

```bash
sudo apt-get install -y aarch64-linux-musl-gcc || true
```

### Host Windows (nativo)

Use toolchain MSVC no proprio Windows:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

## 4) Verificacao rapida de ambiente

### Linux

```bash
command -v aarch64-linux-gnu-gcc
command -v x86_64-w64-mingw32-gcc
command -v musl-gcc
command -v aarch64-linux-musl-gcc
```

### Windows

```powershell
rustup show
```

## 5) Comandos de build

### Script multi-target (recomendado)

Build release com linking dinamico:

```bash
./scripts/build-all.sh --release --linking dynamic
```

Build release com linking estatico:

```bash
./scripts/build-all.sh --release --linking static
```

Selecionando como o script compila o alvo de Windows:

```bash
# resolve automatico: Linux -> windows-gnu, Windows -> windows-msvc
./scripts/build-all.sh --release --windows-target auto

# nativo por host: Linux usa gnu, Windows usa msvc
./scripts/build-all.sh --release --windows-target native

# forcado
./scripts/build-all.sh --release --windows-target gnu
# msvc em Linux gera erro explicito (esperado)
./scripts/build-all.sh --release --windows-target msvc
```

Observacoes sobre `--windows-target`:

- `msvc` exige host Windows (o script falha com erro claro em host Linux).
- `--linking static|dynamic` afeta o alvo `windows-gnu` via rustflags.
- Em host Windows, o script pula os targets Linux automaticamente e executa fluxo nativo de build para Windows.

### Builds individuais (aliases do Cargo)

Linux x64 (GNU):

```bash
cargo build-x64
cargo build-x64-release
```

Linux arm64 (GNU):

```bash
cargo build-arm64
cargo build-arm64-release
```

Windows x64 (GNU / cross):

```bash
cargo build-win64
cargo build-win64-release
```

Windows x64 (MSVC / nativo no Windows):

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

## 6) Caminhos de saida

- Linux x64 dinamico: `target/x86_64-unknown-linux-gnu/release/mdcraft`
- Linux arm64 dinamico: `target/aarch64-unknown-linux-gnu/release/mdcraft`
- Linux x64 estatico: `target/x86_64-unknown-linux-musl/release/mdcraft`
- Linux arm64 estatico: `target/aarch64-unknown-linux-musl/release/mdcraft`
- Windows x64 GNU: `target/x86_64-pc-windows-gnu/release/mdcraft.exe`
- Windows x64 MSVC: `target/x86_64-pc-windows-msvc/release/mdcraft.exe`

## 7) Notas importantes

- Em Linux, mesmo com linking estatico do binario Rust, app GUI ainda pode depender de bibliotecas graficas do sistema alvo.
- Para Windows GNU, o script alterna CRT estatico/dinamico via `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS`.
- Para Windows MSVC, a estrategia de CRT e gerenciada pela toolchain MSVC.

## 8) Troubleshooting

- Erro `linker not found`: instale a toolchain correspondente e valide com `command -v`.
- Falha no alvo `aarch64-unknown-linux-musl`: garanta `aarch64-linux-musl-gcc` no `PATH`.
- Falha no alvo `x86_64-pc-windows-gnu`: garanta `x86_64-w64-mingw32-gcc` instalado.
- Erro ao usar `--windows-target msvc` no Linux: esperado, use host Windows para MSVC ou mude para `gnu`.
