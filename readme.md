# wh

A `which` replacement.

# Features
-	On Windows, automatically appends any extension from `$PATHEXT` if none is provided and match is a file.
-	Full globbing support.
-	Can recursively search directories under `$PATH` if desired.
-	Can limit search to plain files, directories or both.

# Installation
```sh
git clone https://github.com/insomnimus/wh
cd wh
cargo install --path . --locked
```
