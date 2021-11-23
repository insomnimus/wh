use std::{
	env,
	error::Error,
};

use globset::{
	GlobBuilder,
	GlobSetBuilder,
};
use ignore::{
	DirEntry,
	WalkBuilder,
};

use crate::app::{
	Cmd,
	FileType,
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

#[cfg(windows)]
fn is_glob(s: &str) -> bool {
	s.contains(|c| c == '*' || c == '?') || (s.contains('{') && s.contains('}'))
}

impl Cmd {
	#[cfg(windows)]
	fn matches(&self, s: &str, e: &DirEntry, pathext: &[&str]) -> bool {
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
			e.path().file_stem().map_or(false, |stem| eq!(stem, s))
				&& e.path()
					.extension()
					.map_or(true, |ext| pathext.iter().any(|x| eq!(ext, *x)))
		}
	}

	pub fn run(&self) -> Result<(), Box<dyn Error>> {
		let mut n_found = 0_usize;
		let mut set = GlobSetBuilder::new();
		#[cfg(windows)]
		let mut non_globs = Vec::new();

		for s in &self.args {
			#[cfg(not(windows))]
			set.add(
				GlobBuilder::new(s)
					.case_insensitive(!self.respect_case)
					.literal_separator(true)
					.build()?,
			);

			#[cfg(windows)]
			if self.exact || !is_glob(s) {
				non_globs.push(s.as_str());
			} else {
				set.add(
					GlobBuilder::new(s)
						.case_insensitive(!self.respect_case)
						.literal_separator(true)
						.build()?,
				);
			}
		}

		let mut found = vec![false; self.args.len()];
		let set = set.build()?;
		let mut buf = Vec::with_capacity(set.len());

		let paths = env::var("PATH")
			.map_err(|e| format!("could not read the $PATH environment variable: {}", e))?;
		let paths = env::split_paths(&paths).collect::<Vec<_>>();
		#[cfg(windows)]
		let pathext = env::var("PATHEXT").unwrap_or_else(|_| String::from(".exe"));
		#[cfg(windows)]
		let pathext = pathext
			.split(';')
			.filter_map(|s| s.strip_prefix('.'))
			.collect::<Vec<_>>();

		let mut walker = WalkBuilder::new(&paths[0]);
		for p in &paths[1..] {
			walker.add(p);
		}

		let walker = walker
			.standard_filters(false)
			.hidden(self.hidden)
			.follow_links(false)
			.max_depth(self.depth)
			.build();

		for entry in walker.filter_map(|res| match res {
			Ok(entry) if self.file_type.is_match(&entry) => Some(entry),
			Err(e) if !self.quiet => {
				eprintln!("error: {}", e);
				None
			}
			_ => None,
		}) {
			let name = match entry.path().file_name() {
				Some(name) => name,
				None => continue,
			};
			let mut this_found = false;
			if !set.is_empty() {
				set.matches_into(name, &mut buf);
				for idx in &buf {
					found[*idx] = true;
				}
				this_found = !buf.is_empty();
			}

			#[cfg(windows)]
			for (i, s) in non_globs.iter().enumerate() {
				if self.matches(s, &entry, &pathext) {
					this_found = true;
					found[i + set.len()] = true;
				}
			}

			if this_found {
				n_found += 1;
				println!("{}", entry.path().display());
				if self.n != 0 && n_found >= self.n {
					break;
				}
			}
		}

		if self.n == 0 || (n_found < self.n || !found.iter().all(|t| *t)) {
			for s in found.into_iter().enumerate().filter_map(|(i, found)| {
				if found {
					None
				} else {
					self.args.get(i)
				}
			}) {
				eprintln!("{}: not found in $PATH", s);
			}
		}

		Ok(())
	}
}
