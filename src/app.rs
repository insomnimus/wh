use clap::{
	arg,
	crate_version,
	App,
	Arg,
	ArgSettings,
};

#[cfg(windows)]
const EXT_ABOUT: &str = "A ';' separated list of file extensions to add if a file is not given an extension and there is no match.";
#[cfg(windows)]
const PATH_DELIMITER: char = ';';
#[cfg(not(windows))]
const EXT_ABOUT: &str = "A ':' separated list of file extensions to add if a file is not given an extension and there is no match.";
#[cfg(not(windows))]
const PATH_DELIMITER: char = ':';

#[derive(Copy, Clone, PartialEq)]
pub enum FileType {
	File,
	Directory,
	Any,
}

fn validate_usize(s: &str) -> Result<(), String> {
	s.parse::<usize>()
		.map(|_| {})
		.map_err(|_| String::from("the value must be a non-negative integer"))
}

pub struct Cmd {
	pub file_type: FileType,
	pub exact: bool,
	pub n: usize,
	pub respect_case: bool,
	pub depth: Option<usize>,
	pub hidden: bool,
	pub quiet: bool,
	pub args: Vec<String>,
	pub no_auto_ext: bool,
	pub pathext: Vec<String>,
}

impl Cmd {
	fn app() -> App<'static> {
		App::new("wh")
			.about("Find files under $PATH")
			.version(crate_version!())
			.args(&[
				Arg::new("type")
					.help("The file type to look for.")
					.short('t')
					.long("type")
					.default_value("file")
					.possible_values(&["f", "d", "a", "file", "dir", "any"])
					.setting(ArgSettings::IgnoreCase),
				arg!(-e --exact "Do not treat any argument as a glob pattern."),
				arg!(-d --depth <DEPTH> "The search depth. 0 Means no limit.")
					.validator(validate_usize)
					.default_value("1"),
				arg!(n: -n <N> "Show first N results. 0 = show all.")
					.default_value("1")
					.validator(validate_usize),
				arg!(-c --respect-case "Do a case sensitive search."),
				arg!(-X --no-auto-ext "Do not try to match the values from PATH_EXT."),
				arg!(-v --verbose "Report errors."),
				arg!(-a --hidden "Do not ignore hidden directories when recursing."),
				Arg::new("ext")
					.help(EXT_ABOUT)
					.short('x')
					.long("ext")
					.alias("extension")
					.env("PATHEXT")
					.hide_env_values(true)
					.value_delimiter(PATH_DELIMITER),
				Arg::new("args")
					.help("Name/glob pattern to search for.")
					.required(true)
					.multiple_values(true)
					.value_name("NAME OR GLOB"),
			])
	}

	pub fn from_args() -> Self {
		let m = Self::app().get_matches();

		let args: Vec<_> = m.values_of("args").unwrap().map(String::from).collect();
		let file_type = match &m.value_of("type").unwrap().to_lowercase()[..] {
			"f" | "file" => FileType::File,
			"d" | "dir" => FileType::Directory,
			_ => FileType::Any,
		};
		let pathext: Vec<_> = m
			.values_of("ext")
			.map(|it| {
				it.map(|s| s.strip_prefix('.').unwrap_or(s).to_string())
					.collect()
			})
			.unwrap_or_default();

		let quiet = !m.is_present("verbose");
		let respect_case = m.is_present("respect-case");
		let depth = m
			.value_of("depth")
			.map(|s| s.parse::<usize>().unwrap())
			.filter(|n| *n != 0);
		let n = m.value_of("n").unwrap().parse::<usize>().unwrap();

		let hidden = m.is_present("hidden");
		let exact = m.is_present("exact");

		let no_auto_ext = m.is_present("no-auto-ext");

		Self {
			hidden,
			exact,
			pathext,
			n,
			quiet,
			respect_case,
			depth,
			args,
			file_type,
			no_auto_ext,
		}
	}
}
