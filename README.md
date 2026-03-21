# Mdcraft

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![UI](https://img.shields.io/badge/UI-egui%20%2F%20eframe-5A45FF.svg)](https://github.com/emilk/egui)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-1f8b4c.svg)](#build)

Aplicativo desktop para calcular custo, receita e lucro de crafts, com sincronizacao de precos via PXG Wiki e gerenciamento de receitas salvas.

## Visao Geral

O `Mdcraft` foi feito para um fluxo rapido de precificacao:

1. Colar ou editar uma receita.
2. Preencher o preco por item.
3. Comparar item a item com preco NPC.
4. Fechar o craft com custo, receita, lucro e margem.
5. Salvar, importar e exportar receitas em JSON.

## Principais Recursos

- Parse de receita em texto para grade de itens.
- Comparacao visual de preco informado vs preco NPC (com regras fixas).
- Regras fixas de preco NPC para itens especificos de negocio.
- Sincronizacao da wiki em background (sem travar a UI) + atualizacao de crafts.
- Auto-sync diario apos `07:40` (no maximo 1x/dia).
- Opcao de "Preco por item" para producoes multiplas (quantidade > 1 ou craft com `(Nx)`).
- Autosave com debounce para receitas ativas.
- Persistencia local de configuracoes, receitas e precos por item (SQLite + JSON import/export).
- Receitas salvas persistidas em SQLite entre execucoes.

## Atalhos

- `Ctrl + E`: toggle da sidebar (abre/fecha).
- `Esc`: fecha popups (importar, exportar, excluir receita).
- `Enter`: confirma salvamento quando o popup de nome da receita esta ativo.

## Como Rodar

### Requisitos

- Rust toolchain atualizado (`rustup`, `cargo`).
- Linux ou Windows.

No Linux (Ubuntu/Debian), para evitar erros de linker em build cross:

```bash
sudo apt-get update
sudo apt-get install -y gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu gcc-mingw-w64-x86-64 musl-tools
```

No Linux (Arch), instale os equivalentes:

```bash
sudo pacman -Syu --needed aarch64-linux-gnu-gcc aarch64-linux-gnu-binutils mingw-w64-gcc musl
```

No Linux (Fedora), instale os equivalentes:

```bash
sudo dnf install -y gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu mingw64-gcc musl-gcc
```

Se algum pacote tiver nome diferente na sua distro, valide os linkers com:

```bash
command -v aarch64-linux-gnu-gcc
command -v x86_64-w64-mingw32-gcc
command -v musl-gcc
```

### Desenvolvimento

```bash
cargo run
```

### Testes

```bash
cargo test
```

## Build

### Build local padrao

```bash
cargo build --release
```

### Compilacao no Windows

#### Windows nativo (recomendado)

Use toolchain MSVC no proprio Windows (PowerShell):

```powershell
rustup default stable-x86_64-pc-windows-msvc
cargo build --release
```

Saida esperada:

- `target\release\mdcraft.exe`

#### Windows alvo GNU (opcional)

Se voce precisar gerar para `x86_64-pc-windows-gnu` no Windows, instale uma toolchain MinGW compativel e rode:

```powershell
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Saida esperada:

- `target\x86_64-pc-windows-gnu\release\mdcraft.exe`

### Build multi-target (script)

Este script funciona em host Linux e host Windows:

- Linux: build Linux x64/arm64 + Windows (cross).
- Windows: build apenas Windows (fluxo nativo).

```bash
./scripts/build-all.sh --release --linking dynamic
# ou
./scripts/build-all.sh --release --linking static
```

Selecionando o target de Windows no script:

```bash
# Linux: auto/native => windows-gnu (cross)
./scripts/build-all.sh --release --windows-target native

# Forcar windows-gnu
./scripts/build-all.sh --release --windows-target gnu

# Windows: auto/native => windows-msvc (nativo)
./scripts/build-all.sh --release --windows-target auto
./scripts/build-all.sh --release --windows-target msvc
```

### Build nativo para GitHub Releases (Linux + Windows)

O workflow `.github/workflows/release-native-builds.yml` compila binarios nativos em runners do GitHub:

- Linux x64 GNU: `mdcraft-linux-x64.tar.gz`
- Windows x64 MSVC: `mdcraft-windows-x64.zip`

Como usar:

1. `workflow_dispatch`: rode manualmente em **Actions** para gerar artefatos.
2. `release published`: ao publicar uma Release no GitHub, os dois arquivos sao anexados automaticamente na aba de assets.

### Comandos por target (aliases)

```bash
cargo build-x64-release
cargo build-arm64-release
cargo build-win64-release
```

Detalhes completos de targets, toolchains e saidas: `docs/build-targets.md`.

## Sincronizacao de Dados Wiki

- Botao da sidebar: `Sincronizar precos`.
- A sincronizacao manual pode ser disparada a qualquer momento.
- Auto-sync segue regra diaria: apenas apos `07:40` e no maximo 1 vez por dia.
- Ao concluir, o app tambem atualiza o cache de crafts em segundo plano.

## Importacao e Exportacao (JSON)

- `Importar receitas (JSON)`: sempre disponivel.
- `Exportar receitas (JSON)`: aparece quando existem receitas salvas.
- Export suporta JSON grande com area rolavel no popup.

## Estrutura do Projeto

```text
src/
  app/
    sidebar/
    ui_sections/
  data/
    wiki_scraper/
  main/
  model/
  parse/
  units/
  main.rs
scripts/
  build-all.sh
docs/
  build-targets.md
```

## Troubleshooting Rapido

- `linker ... not found`: instale os pacotes de toolchain cross (veja secao de requisitos e `docs/build-targets.md`).
- Build de Windows falhando no Linux: valide `x86_64-w64-mingw32-gcc` no `PATH`.
- Build `aarch64-unknown-linux-musl` falhando: valide `aarch64-linux-musl-gcc` no `PATH`.

## Licenca

MIT
