use std::{
	env,
	fs,
	io::{
		self,
		Read,
		Write,
	},
	os::unix::fs::PermissionsExt,
	path::Path,
};

use atty::Stream;
use clap::Parser;
use globset::{
	GlobBuilder,
	GlobMatcher,
};

use super::{
	shell::{
		self,
		Alias,
		Function,
	},
	App,
	PathEntry,
};

#[derive(Parser)]
pub struct NonWindowsArgs {
	/// For compatibility with GNU which; has no effect
	#[arg(long = "tty-only")]
	_tty_only: bool,

	/// Read aliases from stdin, reporting matching ones on stdout
	#[arg(short = 'i', long)]
	read_alias: bool,

	/// Ignore option --read-alias
	#[arg(long)]
	skip_alias: bool,

	/// Read shell function definitions from stdin, reporting matching ones on
	/// stdout
	#[arg(long)]
	read_functions: bool,

	/// Ignore option --read-functions; don't read stdin
	#[arg(long)]
	skip_functions: bool,
}

enum Item<'a> {
	Path(PathEntry),
	Alias(Alias<'a>),
	Function(Function<'a>),
}

fn new_glob(s: &str) -> (GlobMatcher, bool) {
	let mut is_glob = true;
	let m = GlobBuilder::new(s)
		.backslash_escape(true)
		.build()
		.unwrap_or_else(|_| {
			is_glob = false;
			let mut escaped = String::with_capacity(128);
			for c in s.chars() {
				if matches!(c, '\\' | '[' | ']' | '{' | '}') {
					escaped.push('\\');
				}
				escaped.push(c);
			}

			GlobBuilder::new(&escaped)
				.backslash_escape(true)
				.build()
				.unwrap_or_else(|e| {
					eprintln!("error: failed to escape wildcard symbols for {s}\ncause: {e}\nthis is a big");
					std::process::exit(1);
				})
		})
		.compile_matcher();

	(
		m,
		is_glob && s.contains(|c: char| matches!(c, '*' | '?' | '[' | '{')),
	)
}

fn is_executable(p: &Path) -> bool {
	let Ok(md) = fs::metadata(p) else {
		return false;
	};
	md.file_type().is_file() && md.permissions().mode() & 0o111 != 0
}

impl App {
	pub fn run(self) -> i32 {
		let home = home::home_dir();
		let pwd = if self.show_dot {
			env::current_dir().ok()
		} else {
			None
		};

		let mut buf;
		let mut items = Vec::with_capacity(256);
		if (self.x.read_alias && !self.x.skip_alias)
			|| (self.x.read_functions && !self.x.skip_functions)
		{
			if atty::is(Stream::Stdin) {
				let warn = if self.x.read_alias
					&& !self.x.skip_alias
					&& self.x.read_functions
					&& !self.x.skip_functions
				{
					"--read-functions, --read-alias, -i"
				} else if self.x.read_alias && !self.x.skip_alias {
					"--read-alias, -i"
				} else {
					"--read-functions"
				};
				eprintln!("wh: {warn}: Warning: stdin is a tty.");
			}
			buf = String::with_capacity(4096);
			let _ = io::stdin().lock().read_to_string(&mut buf);
			let xs = shell::parse(&buf);
			items.reserve(xs.len());
			items.extend(xs.into_iter().map(|i| match i {
				shell::Item::Function(x) => Item::Function(x),
				shell::Item::Alias(x) => Item::Alias(x),
			}));
			// Aliases have priority
			items.sort_by_key(|i| match i {
				Item::Alias(_) => 0u8,
				Item::Function(_) => 1u8,
				Item::Path(_) => 2u8,
			});
		}

		let path = env::var_os("PATH").unwrap_or_default();
		items.extend(
			env::split_paths(&path)
				.filter(|p| {
					!p.as_os_str().is_empty()
						&& p.to_str().map_or(true, |s| !s.trim().is_empty())
						&& (!self.skip_dot
							|| p.components().all(|c| {
								!c.as_os_str().to_str().unwrap_or_default().starts_with('.')
							})) && (!self.skip_tilde
						|| (!p.starts_with("~")
							&& home.as_deref().map_or(true, |home| !p.starts_with(home))))
				})
				.map(|p| {
					if !self.skip_tilde {
						Item::Path(PathEntry::new(
							home.as_deref()
								.and_then(|home| {
									p.strip_prefix("~").ok().map(|rest| home.join(rest))
								})
								.unwrap_or(p),
						))
					} else {
						Item::Path(PathEntry::new(p))
					}
				}),
		);

		let mut not_found = 0;
		let mut stdout = io::stdout().lock();
		'outer: for c in &self.commands {
			let (glob, is_glob) = new_glob(c);
			let mut found = false;

			for i in &mut items {
				let yes = match i {
					Item::Alias(a)
						if (self.x.read_alias && !self.x.skip_alias) && glob.is_match(a.lhs) =>
					{
						let _ = writeln!(stdout, "{a}");
						true
					}
					Item::Function(f)
						if (self.x.read_functions && !self.x.skip_functions)
							&& glob.is_match(f.name) =>
					{
						let _ = writeln!(stdout, "{f}");
						true
					}
					Item::Path(p) => {
						let mut yes = false;
						for file in p.get() {
							// unwrapping file_name is okay here because PathEntry::get pre-filters
							if glob.is_match(file.file_name().unwrap()) && is_executable(file) {
								self.write_path(&mut stdout, home.as_deref(), pwd.as_deref(), file);
								yes = true;
								found = true;
								if !is_glob {
									break;
								}
							}
						}
						yes
					}
					_ => false,
				};

				found = found || yes;

				if yes && !self.all {
					continue 'outer;
				}
			}

			if !found {
				eprintln!("{c}: not found");
				not_found += 1;
			}
		}

		not_found
	}
}
