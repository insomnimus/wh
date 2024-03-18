# wh

A `which` replacement.

## Features
- On Windows: Check for missing extensions from the `PATHEXT` environment variable as well as `.exe`
-	Linux style globbing support.
- On non-Windows: Read function and alias definitions from stdin just like GNU which.
- On non-Windows: Mostly a superset of GNU which.

## Installation
### Installation On Windows Via [Scoop](https://github.com/ScoopInstaller/Scoop)
This way you can update wh with scoop.
1. Add my [personal bucket](https://github.com/insomnimus/scoop-bucket) to Scoop.\
	`scoop bucket add insomnia https://github.com/insomnimus/scoop-bucket`
2. Update scoop.\
	`scoop update`
3. Install wh.\
	`scoop install wh`

### Other Methods
Download a binary for your platform from [the releases page](releases/)

Or build it from source:

```sh
cargo install --locked --branch main --git https://github.com/insomnimus/wh
```

### Suggested Usage On Non-Windows Platforms
You might want to add a shell function in your profile so that `wh` can read your aliases and functions:
```shell
wh() {
	{
		alias
		declare -f
	} | /usr/bin/wh --read-alias --read-functions "$@"
}
```

Don't forget to change `/usr/bin/wh` with the full path of `wh` on your system.
