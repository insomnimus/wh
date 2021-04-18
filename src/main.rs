use std::env;
use std::path::Path;
use std::process;

fn show_help() {
    let cmd = match env::current_exe() {
        Ok(p) => match p.file_name() {
            Some(s) => s.to_str().unwrap().to_owned(),
            None => String::from("which"),
        },
        Err(_) => String::from("which"),
    };

    eprintln!(
        "{}, finds executables under $PATH
usage:
	{} [options] <filename>
options:
	-a, --all: print every match instead of just the first
	-e, --exact: do not append missing '.exe' prefix
	-h, --help: show this message and exit",
        cmd, cmd
    );
    std::process::exit(0);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() <= 1 {
        show_help();
    }

    let mut target = "";
    let mut flag_all = false;
    let mut flag_exact = false;

    for a in &args[1..] {
        match &a[..] {
            "-h" | "--help" => show_help(),
            "-a" | "--all" => flag_all = true,
            "-e" | "--exact" => flag_exact = true,
            "-ea" | "-ae" => {
                flag_all = true;
                flag_exact = true;
            }
            _ => {
                if target != "" {
                    eprintln!("too many arguments, run witth --help for the usage");
                    process::exit(2);
                }
                target = &a[..];
            }
        }
    }
    if target == "" && (flag_all || flag_exact) {
        eprintln!("missing argument: command to search for");
        process::exit(2);
    }
    if target == "" {
        show_help();
    }

    let target = if flag_exact || target.ends_with(".exe") {
        target.to_owned()
    } else {
        format!("{}.exe", target)
    };

    let mut found = false;
    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                if Path::new(&format!("{}/{}", path.display(), &target)).exists() {
                    println!("{}\\{}", path.display(), &target);
                    if !flag_all {
                        return;
                    }
                    found = true;
                }
            }
        }
        None => panic!("can't get the value of $PATH"),
    }
    if found {
        return;
    }
    eprintln!("{}: not found", target);
    process::exit(1);
}
