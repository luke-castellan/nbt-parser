# NBT-Parser CLI
This NBT-Parser let you **view your NBT-Files** in your command line.

## Usage
Most of the normal NBT Files are **compressed with GZip** by default (e.g. level.dat, ...). Remember to use the `--gzip` or `--g` flag in that case!
```bash
nbt-parser (--gzip) <FILE>
```
### Installation as a CLI tool
It is recommended to **build from source**. The **Rust Compiler** and **Cargo Package Manager** is necessary for this part. 
Both can be downloaded with the [install script](https://www.rust-lang.org/tools/install).
Alternatively, you can download the latest release.
1. Clone the repository to your computer
```bash
git clone https://github.com/luke-castellan/nbt-parser.git
```
2. Go into the directory
```bash
cd nbt-parser
```

3. Install it to your path with `cargo install`
```bash
cargo install --path .
```
