use clap::{crate_version, App, AppSettings, Arg};

pub fn new() -> App<'static> {
    let app= App::new("wh")
	.version(crate_version!())
	.about("find files")
	.long_about("find files under $PATH or search under a directory")
	.setting(AppSettings::UnifiedHelpMessage)
	.help_template(
"wh, {about}
usage:
{bin} [OPTIONS] <PATTERN...>

{unified}
	{after-help}
",
	)
	.after_long_help("the default behaviour is to look under $PATH
if the search string contains a wildcard and the --exact flag is not set, the --all flag will be assumed to be set");

    let no_check = Arg::new("no-check")
        .long("no-check")
        .short('n')
        .about("do not ignore patterns containing only '*'")
        .conflicts_with("exact");

    let file_type_filter = Arg::new("type")
        .short('t')
        .long("type")
        .takes_value(true)
        .possible_values(&["any", "file", "folder"])
        .about("filter the search by file type");

    let all = Arg::new("all")
        .long("all")
        .short('a')
        .about("do not stop after the first result, print them all")
        .takes_value(false);

    let exact = Arg::new("exact")
        .short('e')
        .long("exact")
        .takes_value(false);

    #[cfg(windows)]
    let exact = exact.about("do not interprete glob patterns and do not append missing .exe");
    #[cfg(not(windows))]
    let exact = exact.about("do not interprete glob patterns");

    let recursive = Arg::new("recursive")
        .short('r')
        .long("recursive")
        .about("search $PATH recursively")
        .conflicts_with("find-under")
        .takes_value(false);

    let find_under = Arg::new("find-under")
        .short('f')
        .long("find-under")
        .about("recursively search under a directory ")
        .multiple(true)
        .takes_value(true);

    #[cfg(windows)]
    let no_auto_exe = Arg::new("no-auto-exe")
        .about("do not append missing .exe extension")
        .short('N')
        .long("no-auto-exe")
        .takes_value(false);

    let args = Arg::new("target")
        .takes_value(false)
        .multiple(true)
        .about("file or glob pattern to search for")
        .required(true);

    let hidden = Arg::new("hidden")
        .short('d')
        .long("hidden")
        .about("do not ignore hidden directories")
        .takes_value(false);

    let app = app
        .arg(all)
        .arg(exact)
        .arg(no_check)
        .arg(hidden)
        .arg(recursive)
        .arg(file_type_filter)
        .arg(find_under)
        .arg(args);

    #[cfg(windows)]
    return app.arg(no_auto_exe);
    #[cfg(not(windows))]
    return app;
}
