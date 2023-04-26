# wh

A `which` replacement.

# Features
- On Windows: Check for missing extensions from the `PATHEXT` environment variable as well as `.exe`
-	Linux style globbing support.
- On non-Windows: Read function and alias definitions from stdin just like GNU which.
- On non-Windows: Mostly a superset of GNU which.

# Installation
Download a binary for your platform from [the releases page](releases/)

Or build it from source:

```sh
cargo install --locked --branch main --git https://github.com/insomnimus/wh
```
