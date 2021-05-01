use std::env;
use std::path::PathBuf;
use std::process::exit;

fn show_usage() {
    let cmd = match env::current_exe() {
        Ok(p) => match p.file_name() {
            Some(s) => s.to_str().unwrap().to_owned(),
            None => String::from("wh"),
        },
        Err(_) => String::from("wh"),
    };
    eprintln!(
        "usage:
	{} [options] <filename...>",
        cmd
    );
    exit(1);
}

fn show_help() {
    let cmd = match env::current_exe() {
        Ok(p) => match p.file_name() {
            Some(s) => s.to_str().unwrap().to_owned(),
            None => String::from("wh"),
        },
        Err(_) => String::from("wh"),
    };

    println!(
        "{}, finds files under $PATH
usage:
	{} [options] <filename...>
options:
	-a, --all: print every match instead of just the first",
        cmd, cmd
    );
    #[cfg(windows)]
    println!("\t-e, --exact: do not append missing '.exe' prefix");

    exit(0);
}

struct CmdArgs {
    f_help: bool,
    f_all: bool,
    show_usage: bool,
    targets: Vec<String>,
}

impl CmdArgs {
    fn parse(args: Vec<String>) -> Self {
        if args.is_empty() {
            return Self {
                show_usage: true,
                f_help: false,
                f_all: false,
                targets: args,
            };
        }

        let mut targets: Vec<String> = vec![];
        let mut f_all = false;
        #[cfg(windows)]
        let mut f_exact = false;
        for a in args {
            match &a[..] {
                "-h" | "--help" => {
                    return Self {
                        f_help: true,
                        f_all: false,
                        targets: vec![],
                        show_usage: false,
                    }
                }
                "-a" | "--all" => f_all = true,
                #[cfg(windows)]
                "-ea" | "-ae" => {
                    f_all = true;
                    f_exact = true;
                }
                #[cfg(windows)]
                "-e" | "--exact" => f_exact = true,
                _ => targets.push(a),
            };
        }

        #[cfg(windows)]
        if f_exact {
            return Self {
                f_all,
                f_help: false,
                show_usage: false,
                targets,
            };
        } else {
            targets = targets
                .into_iter()
                .map(|x| if x.ends_with(".exe") { x } else { x + ".exe" })
                .collect();
        }

        Self {
            f_help: false,
            f_all,
            targets,
            show_usage: false,
        }
    }

    fn execute(&self) -> i32 {
        let paths = env::var("PATH").unwrap_or_else(|e| {
            eprintln!("could not access the value of $PATH: {:?}", e);
            exit(1);
        });

        let paths: Vec<PathBuf> = env::split_paths(&paths).collect();

        let mut exit_code = 0i32;
        for t in &self.targets {
            let mut found = false;
            for p in &paths {
                let mut x = p.clone();
                x.push(&t);
                if x.exists() {
                    found = true;
                    println!("{}", x.display());
                    if !self.f_all {
                        break;
                    }
                }
            }
            if !found {
                println!("{}: not found", &t);
                exit_code = 1;
            }
        }

        exit_code
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = CmdArgs::parse(args);
    if cmd.f_help {
        show_help();
        return;
    }
    if cmd.show_usage {
        show_usage();
        return;
    }
    if cmd.targets.is_empty() {
        eprintln!("missing required argument: target to search for");
        exit(1);
    }
    exit(cmd.execute());
}
