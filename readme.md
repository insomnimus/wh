# wh

A `which` replacement.

# Features
-	Check for missing extensions if none is given and the match is a file. The values are retreived from the `$PATHEXT` env variable, which is set by default on Windows and contains extensions like `.exe` and `.bat`.
-	Full globbing support.
-	Can recursively search directories under `$PATH` if desired.
-	Can limit search to plain files, directories or both.

# Installation
```sh
git clone https://github.com/insomnimus/wh
cd wh
cargo install --path . --locked
```
