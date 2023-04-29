#[cfg(not(windows))]
mod non_win;
#[cfg(not(windows))]
mod shell;
#[cfg(windows)]
mod win;

use std::{
	fs::{self,},
	io::{
		StdoutLock,
		Write,
	},
	path::{
		self,
		Path,
		PathBuf,
	},
};

use clap::Parser;

#[derive(Parser)]
/// Write the full path of COMMAND(s) to standard output.
#[command(version)]
struct App {
	/// Skip directories in PATH that start with a dot.
	#[arg(long)]
	skip_dot: bool,

	/// Skip directories in PATH that start with a tilde and executables which
	/// reside in the HOME directory.
	#[arg(long)]
	skip_tilde: bool,

	/// If a directory in PATH starts with a dot and a matching executable was
	/// found for that path, then print "./programname" rather than the full
	/// path.
	#[arg(long)]
	show_dot: bool,

	/// Output a tilde when a directory matches the HOME directory. This option
	/// is ignored when wh is invoked as root (admin on Windows).
	#[arg(long)]
	show_tilde: bool,

	/// Print all matches, not just the first.
	#[arg(short, long)]
	all: bool,

	/// Commands to look in PATH.
	#[arg(required = true, value_name = "command")]
	commands: Vec<String>,

	#[cfg(windows)]
	#[cfg_attr(windows, command(flatten))]
	x: win::WindowsArgs,

	#[cfg(not(windows))]
	#[cfg_attr(not(windows), command(flatten))]
	x: non_win::NonWindowsArgs,
}

impl App {
	fn write_path(
		&self,
		stdout: &mut StdoutLock,
		home: Option<&Path>,
		pwd: Option<&Path>,
		path: &Path,
	) {
		if self.show_tilde {
			if let Some(suffix) = home.and_then(|home| path.strip_prefix(home).ok()) {
				let _ = writeln!(stdout, "~{}{}", path::MAIN_SEPARATOR, suffix.display());
				return;
			}
		}

		if self.show_dot {
			if let Some(suffix) = pwd.and_then(|pwd| path.strip_prefix(pwd).ok()) {
				let _ = writeln!(stdout, ".{}{}", path::MAIN_SEPARATOR, suffix.display());
				return;
			}
		}

		let _ = writeln!(stdout, "{}", path.display());
	}
}

struct PathEntry {
	path: PathBuf,
	files: Option<Vec<PathBuf>>,
}

impl PathEntry {
	fn new(path: PathBuf) -> Self {
		Self { path, files: None }
	}

	fn get(&mut self) -> &[PathBuf] {
		self.files.get_or_insert_with(|| {
			let Ok(entries) = fs::read_dir(&self.path) else {
				return Vec::new();
			};
			entries
				.filter_map(|e| {
					let e = e.ok()?;
					let ftype = e.file_type().ok()?;
					#[cfg(windows)]
					{
						use std::os::windows::fs::FileTypeExt;
						if !ftype.is_file() && !ftype.is_symlink_file() {
							return None;
						}
					}
					#[cfg(not(windows))]
					if ftype.is_dir() {
						return None;
					}

					let p = e.path();
					if p.file_name().is_none() {
						None
					} else {
						Some(p)
					}
				})
				.collect()
		})
	}
}

fn main() {
	let mut app = App::parse();
	if app.show_tilde {
		#[cfg(not(windows))]
		{
			app.show_tilde = &whoami::username() != "root";
		}

		#[cfg(windows)]
		unsafe {
			app.show_tilde = !windows::Win32::UI::Shell::IsUserAnAdmin().as_bool();
		}
	}

	for c in &app.commands {
		if (cfg!(windows) && c.contains(|c: char| c == '\\' || c == '/'))
			|| (cfg!(not(windows)) && c.contains('/'))
		{
			eprintln!("error: {c}: commands can't contain path separators");
			std::process::exit(1);
		}
	}
	std::process::exit(app.run());
}
