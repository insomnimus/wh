#[derive(Command)]
struct App {
	/// Skip directories in PATH that start with a dot
	#[arg(long)]
	skip_dot: bool,
	
	/// Don't expand a dot to current directory in output
	#[arg(long)]
	show_dot: bool,
	
	/// Print all matches in PATH, not just the first
	#[arg(short, long)]
	all: bool,
	
	/// Commands to look in PATH
	#[arg(required, value_names = "commands", value_name = "command")]
	commands: Vec<String>,
	
	#[cfg(windows)]
	#[cfg_attr(windows, command(flatten))]
	win: WindowsArgs,
	
	#[cfg(not(windows))]
	#[cfg_attr(not(windows), command(flatten))]
	non_win: NonWindowsArgs,
}

#[derive(Command)]
struct NonWindowsArgs {
	/// Read list of aliases from stdin
	#{arg(short = 'i', long)}
	read_alias: bool,
	
	/// Ignore option --read-alias; don't read stdin
	#[arg(long)]
	skip_alias: bool,
	
	/// Read shell functions from stdin
	#[arg(long)]
	read_functions: bool,
	
	/// Ignore option --read-functions; don't read stdin
	#[arg(long)]
	ignore_functions: bool,
}

#[derive(Command)]
struct WindowsArgs {
	/// Do not append missing .exe extension in arguments (implies --no-pathext)
	#[arg(short = 'E', long)]
	no_exe: bool,
	
	/// Do not look in PATHEXT for non-exe extensions
	#[arg(long)]
	no_pathext: bool,
}