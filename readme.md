# wh

A replacement for the `which` tool.

Wh finds files under your `$PATH` and that's all it does.

## Features

-	expand glob patterns.
-	Cross platform.
-	Shipped with shell completions.

## Installation

You will need a recent [rust](https://github.com/rust-lang/rust) setup and [cargo](https://github.com/rust-lang/cargo).

There're currently two ways to install `wh` on your system:

### 1- Install using git clone

```sh
git clone https://github.com/insomnimus/rs-which
cd rs-which
cargo install --path .
```

### 2- Using cargo install

```sh
cargo install --git https://github.com/insomnimus/rs-which --branch main
```

## Usage

After following any of the installation methods above, the `wh` binary will be under one of
-	`~/.cargo/bin`
-	`$CARGO_HOME/bin` (non-windows)
-	`$env:CARGO_HOME\bin` (windows)

So make sure it's under your `$PATH`.

Then just use it as you would the `which` tool:

`wh cargo`

`wh --all 'cargo-*'`
