use std::{
	env,
	io::{
		self,
	},
	path::Path,
};

use clap::Parser;
use globset::{
	Glob,
	GlobBuilder,
};

use super::*;

#[derive(Parser)]
pub struct WindowsArgs {
	/// Do not look in PATHEXT for non-exe extensions
	#[arg(short = 'P', long)]
	no_pathext: bool,
}

fn new_glob(s: &str, suffix: &str) -> (Glob, bool) {
	let mut is_glob = true;
	let concat = format!("{s}{suffix}");
	let glob = GlobBuilder::new(&concat)
		.case_insensitive(true)
		.build()
		.unwrap_or_else(|_| {
			is_glob = false;
			let mut escaped = String::with_capacity(128);
			for c in s.chars() {
				if matches!(c, '[' | ']' | '{' | '}') {
					escaped.push('\\');
				}
				escaped.push(c);
			}

			escaped += suffix;

			GlobBuilder::new(&escaped)
				.case_insensitive(true)
				.backslash_escape(true)
				.build()
				.unwrap_or_else(|e| {
					eprintln!("error: failed to escape wildcard symbols for {s}\ncause: {e}\nthis is a big");
					std::process::exit(1);
				})
		});

	(
		glob,
		is_glob && s.contains(|c: char| matches!(c, '*' | '?' | '[' | '{')),
	)
}

impl App {
	pub fn run(self) -> i32 {
		let home = home::home_dir();
		let pwd = if self.show_dot {
			env::current_dir().ok()
		} else {
			None
		};

		let path = env::var_os("PATH").unwrap_or_default();
		let mut path = env::split_paths(&path)
			.filter(|p| {
				!p.as_os_str().is_empty()
					&& p.to_str().map_or(true, |s| !s.trim().is_empty())
					&& (!self.skip_dot
						|| p.components()
							.all(|c| !c.as_os_str().to_str().unwrap_or_default().starts_with('.')))
					&& (!self.skip_tilde
						|| (!p.starts_with("~")
							&& home.as_deref().map_or(true, |home| !p.starts_with(home))))
			})
			.map(|p| {
				if !self.skip_tilde {
					PathEntry::new(
						home.as_deref()
							.and_then(|home| p.strip_prefix("~").ok().map(|rest| home.join(rest)))
							.unwrap_or(p),
					)
				} else {
					PathEntry::new(p)
				}
			})
			.collect::<Vec<_>>();

		let pathext_;
		let pathext = if self.x.no_pathext {
			vec!["exe"]
		} else {
			pathext_ = env::var("PATHEXT").unwrap_or_else(|_| ".exe".into());
			let mut pathext = pathext_
				.split(';')
				.map(|x| x.strip_prefix('.').unwrap_or(x))
				.filter(|x| !x.is_empty())
				.collect::<Vec<_>>();
			if !pathext.iter().any(|x| x.eq_ignore_ascii_case("exe")) {
				pathext.insert(0, "exe");
			}
			pathext
		};

		let pathext_glob = {
			let mut buf = String::with_capacity(256);
			buf += ".{";
			for ext in &pathext {
				if ext.contains(|c: char| c != '.' && !c.is_alphanumeric()) {
					eprintln!("error: PATHEXT contains an invalid entry ({ext})");
					return 1;
				} else if ext.is_empty() {
					continue;
				}
				if buf.len() != 2 {
					buf.push(',');
				}
				buf += ext;
			}
			buf.push('}');

			buf
		};

		let filter = |p: &Path| -> bool {
			let Some(ext) = p.extension() else {
				return false;
			};
			pathext.iter().any(|e| ext.eq_ignore_ascii_case(e))
		};

		let mut stdout = io::stdout().lock();
		let mut not_found = 0;
		'outer: for c in &self.commands {
			let mut is_glob = false;
			let glob = match c.rsplit_once('.') {
				Some(("", "")) => {
					eprintln!("error: invalid command '.'");
					not_found += 1;
					continue;
				}
				None | Some((_, "")) | Some(("", _)) => {
					let (glob, ig) = new_glob(c, &pathext_glob);
					is_glob = is_glob || ig;
					glob
				}
				Some((_, ext)) => {
					let (glob, ig) = new_glob(
						c,
						if pathext.iter().any(|x| x.eq_ignore_ascii_case(ext))
							|| ext
								.chars()
								.any(|c: char| matches!(c, '?' | '*' | '[' | '{'))
						{
							""
						} else {
							&pathext_glob
						},
					);
					is_glob = is_glob || ig;
					glob
				}
			};

			let glob = glob.compile_matcher();
			let mut found = false;
			for p in &mut path {
				for file in p.get().iter().filter(|p| filter(p)) {
					// unwrapping file_name is okay because PathEntry::get pre-fitlers
					if glob.is_match(file.file_name().unwrap()) {
						self.write_path(&mut stdout, home.as_deref(), pwd.as_deref(), file);
						found = true;
						if !self.all {
							continue 'outer;
						} else if !is_glob {
							break;
						}
					}
				}
			}

			if !found {
				not_found += 1;
				eprintln!("{c}: not found");
			}
		}

		not_found
	}
}
