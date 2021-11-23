use clap::{
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
		let app = App::new("wh")
			.about("Find files under $PATH")
			.version(crate_version!());

		let file_type = Arg::new("type")
			.about("The file type to look for.")
			.short('t')
			.long("type")
			.default_value("file")
			.possible_values(&["f", "d", "a", "file", "dir", "any"])
			.setting(ArgSettings::IgnoreCase);

		let exact = Arg::new("exact")
			.about("Do not treat any argument as a glob pattern.")
			.short('e')
			.long("exact");

		let depth = Arg::new("depth")
			.about("The recursion depth. 0 = no limit.")
			.short('d')
			.long("depth")
			.validator(validate_usize)
			.default_value("1");

		let n = Arg::new("n")
			.about("Show first N matches. 0 = all. Defaults to the number of arguments.")
			.short('n')
			.long("max")
			.takes_value(true)
			.validator(validate_usize);

		let respect_case = Arg::new("respect-case")
			.about("Do not ignore case.")
			.short('c')
			.long("respect-case");

		let no_auto_ext = Arg::new("no-auto-ext")
			.about("Do not add missing extension for files from $PATHEXT.")
			.short('E')
			.long("no-auto-ext");

		let verbose = Arg::new("verbose")
			.about("Report errors.")
			.short('v')
			.long("verbose");

		let args = Arg::new("args")
			.about("The file name or glob pattern to search for.")
			.required(true)
			.value_name("query")
			.setting(ArgSettings::MultipleValues);

		let hidden = Arg::new("hidden")
			.about("Do not ignore hidden directories.")
			.short('a')
			.long("all");

		let ext = Arg::new("ext")
			.about(EXT_ABOUT)
			.short('x')
			.long("ext")
			.alias("extension")
			.env("PATHEXT")
			.hide_env_values(true)
			.value_delimiter(PATH_DELIMITER);

		app.arg(file_type)
			.arg(respect_case)
			.arg(depth)
			.arg(n)
			.arg(verbose)
			.arg(ext)
			.arg(exact)
			.arg(hidden)
			.arg(no_auto_ext)
			.arg(args)
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
		let n = m
			.value_of("n")
			.map(|s| s.parse::<usize>().unwrap())
			.unwrap_or_else(|| args.len());
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
