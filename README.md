# sb2gs

Decompile Scratch projects into [**goboscript**](https://github.com/aspizu/goboscript) 
projects automatically.

### Installation

Use [Cargo](https://doc.rust-lang.org/cargo/) to install sb2gs globally.

```bash
cargo install --git https://github.com/aspizu/sb2gs --package sb2gs-cli
```

### Usage

You can directly download and decompile a Scratch project from its ID.

```bash
sb2gs --id 12345678 my_project_name --verify
```

### Wasm

```bash
wasm-pack build crates/wasm --target web
```

`--verify` requires goboscript to be installed.

```
usage: sb2gs [-h] [--overwrite] [--id ID] [--verify] input [output]

positional arguments:
  input
  output

options:
  -h, --help   show this help message and exit
  --overwrite
  --id ID      Download the project with this ID.
  --verify     Invoke goboscript to verify that the decompiled code is valid. This does not indicate that the decompiled code is equivalent
               to the original.
```
