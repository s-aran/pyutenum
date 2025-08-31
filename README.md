# Pyutenum (rspyast)

[日本語はこちら](README_J.md)

## Overview

Pyutenum is a command‑line tool that enumerates test names written for the Python standard testing framework unittest.

It scans Python script files whose names start with `test` (the standard naming convention for tests) and lists fully‑qualified method names that start with `test\_`. The enumerated test names can then be passed to a test runner for execution.

## Installation

You can download a binary appropriate for your environment from the GitHub releases page.

Alternatively, because Pyutenum is written in Rust, you can build and install it from source if you have a Rust development environment:

```sh
cargo install --path .
```

## Usage

```sh
./rspyast # enumerate tests with the current directory as the base
./rspyast tests # enumerate tests using the "tests" directory as the base
./rspyast --select # interactively select from the list of enumerated tests using the current directory as the base
```

Pyutenum only enumerates tests. It is intended to be used by filtering the test list via `grep` or `fzf` and passing the result to a test runner.

## License

MIT
