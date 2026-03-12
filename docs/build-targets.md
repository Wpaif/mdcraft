# Build targets (x64 / arm64)

## 1) Install Rust targets

```bash
rustup target add \
 x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
 x86_64-unknown-linux-musl aarch64-unknown-linux-musl \
 x86_64-pc-windows-gnu
```

## 2) Install cross toolchains (Linux host)

```bash
sudo apt-get update
sudo apt-get install -y gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu gcc-mingw-w64-x86-64
```

For static Linux builds with musl you also need:

```bash
sudo apt-get install -y musl-tools
# arm64 musl toolchain package name varies by distro; common option:
sudo apt-get install -y aarch64-linux-musl-gcc || true
```

## 3) Build commands

### Dynamic linking (default)

```bash
./scripts/build-all.sh --release --linking dynamic
```

### Static linking

```bash
./scripts/build-all.sh --release --linking static
```

### Individual targets (dynamic)

#### Linux x64

```bash
cargo build-x64
cargo build-x64-release
```

### Linux arm64

```bash
cargo build-arm64
cargo build-arm64-release
```

### Windows x64

```bash
cargo build-win64
cargo build-win64-release
```

## 4) Output paths

- Dynamic Linux x64: `target/x86_64-unknown-linux-gnu/release/mdcraft`
- Dynamic Linux arm64: `target/aarch64-unknown-linux-gnu/release/mdcraft`
- Static Linux x64: `target/x86_64-unknown-linux-musl/release/mdcraft`
- Static Linux arm64: `target/aarch64-unknown-linux-musl/release/mdcraft`
- Windows x64: `target/x86_64-pc-windows-gnu/release/mdcraft.exe`

Notes:

- Linux GUI apps may still require graphics stack/runtime libraries on the target machine, even with static Rust linking.
- On Windows, this setup toggles static/dynamic CRT via rustflags in the build script.
