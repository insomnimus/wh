use std::{
	env,
	error::Error,
};

use globset::GlobBuilder;
use ignore::{
	DirEntry,
	WalkBuilder,
};

use crate::{
	app::{
		Cmd,
		FileType,
	},
	matcher::*,
};

impl FileType {
	fn is_match(self, e: &DirEntry) -> bool {
		match self {
			Self::Any => true,
			Self::File => e.file_type().map_or(false, |t| t.is_file()),
			Self::Directory => e.file_type().map_or(false, |t| t.is_dir()),
		}
	}
}

fn is_glob(s: &str) -> bool {
	s.contains(|c| c == '*' || c == '?') || (s.contains('{') && s.contains('}'))
}

impl Cmd {
	fn matches(&self, m: &Matcher<'_>, e: &DirEntry) -> bool {
		match m {
			Matcher::Str(s) => self.matches_str(s, e),
			Matcher::Glob(g) => e.path().file_name().map_or(false, |name| g.is_match(name)),
		}
	}

	fn matches_str(&self, s: &str, e: &DirEntry) -> bool {
		macro_rules! eq {
			($a:expr, $b:expr) => {
				if self.respect_case {
					$b.eq($a)
				} else {
					$a.eq_ignore_ascii_case($b)
				}
			};
		}

		if self.no_auto_ext
			|| self.file_type == FileType::Directory
			|| e.file_type().map_or(false, |t| t.is_dir())
		{
			e.path().file_name().map_or(false, |name| eq!(name, s))
		} else {
			e.path().file_name().map_or(false, |p| eq!(p, s))
				|| (!self.no_auto_ext
					&& e.path().file_stem().map_or(false, |stem| eq!(stem, s))
					&& e.path().extension().map_or(true, |ext| {
						self.pathext.iter().any(|x| eq!(ext, x.as_str()))
					}))
		}
	}

	pub fn run(&self) -> Result<(), Box<dyn Error>> {
		let mut n_found = 0_usize;
		let mut matches = self
			.args
			.iter()
			.map(|s| {
				if self.exact || !is_glob(s) {
					Ok(Matcher::Str(s.as_str()))
				} else {
					GlobBuilder::new(s)
						.case_insensitive(!self.respect_case)
						.literal_separator(true)
						.build()
						.map(|g| Matcher::Glob(g.compile_matcher()))
				}
				.map(|matcher| MatchState { matcher, count: 0 })
			})
			.collect::<Result<Vec<_>, _>>()?;

		let paths = env::var("PATH")
			.map_err(|e| format!("could not read the $PATH environment variable: {}", e))?;
		let paths = env::split_paths(&paths).collect::<Vec<_>>();

		let mut walker = WalkBuilder::new(&paths[0]);
		for p in &paths[1..] {
			walker.add(p);
		}

		let walker = walker
			.standard_filters(false)
			.hidden(self.hidden)
			.follow_links(true)
			.max_depth(self.depth)
			.build();

		for entry in walker.filter_map(|res| match res {
			Ok(entry) if self.file_type.is_match(&entry) && entry.path().file_name().is_some() => {
				Some(entry)
			}
			Err(e) if !self.quiet => {
				eprintln!("error: {}", e);
				None
			}
			_ => None,
		}) {
			let mut this_matched = false;
			for m in &mut matches {
				if (self.n == 0 || m.count < self.n) && self.matches(&m.matcher, &entry) {
					m.count += 1;
					n_found += 1;
					this_matched = true;
				}
			}

			if this_matched {
				println!("{}", entry.path().display());
				if self.n != 0 && n_found >= self.n * self.args.len() {
					break;
				}
			}
		}

		for m in &matches {
			if m.count == 0 {
				eprintln!("{}: not found under $PATH", &m.matcher);
			}
		}

		Ok(())
	}
}
