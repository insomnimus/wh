use std::env;
use std::path::Path;

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
	{} <filename>",
        cmd, cmd
    );
    std::process::exit(0);
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        show_help();
    }
    let arg = &args[1];

    if arg == "-h" || arg == "--help" {
        show_help();
    }
    let target: String = if str::ends_with(&arg, ".exe") {
        arg.to_owned()
    } else {
        format!("{}.exe", arg).to_owned()
    };

    match env::var_os("PATH") {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                //let p= format!("{}/{}", &path.display(), &target);
                //println!("{}", p);
                if Path::new(&format!("{}/{}", path.display(), &target)).exists() {
                    println!("{}\\{}", path.display(), &target);
                    return;
                }
            }
        }
        None => panic!("can't get the value of $PATH"),
    }
    eprintln!("{}: not found", arg);
    std::process::exit(1);
}
