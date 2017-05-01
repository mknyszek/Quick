# Quick Programming Language

A prototype quantum scripting language, complete with a simple bytecode-based interpreter.

## License

This repository is licensed under GPLv3, because it depends upon
`rust-libquantum`, which is GPLv3 because it ultimately depends on libquantum.

## Installation

`git clone` this repository and run `cargo build --release`.

## Run

Run the command `target/release/qscript < <some file>.qs` to execute a QScript
program.

## Progress

There are currently numerous bugs, and the language is not yet
feature-complete. The current bugs and missing features may be found in the
issues section on this GitHub page.
