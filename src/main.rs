extern crate clap;
extern crate glob;

use clap::{App, AppSettings, Arg};
use glob::{MatchOptions, Pattern};
use std::env;
use std::path::PathBuf;
use std::process::exit;

const VERSION: &str = "0.2.0";
const ABOUT: &str = "find files under $PATH";

struct Cmd {
    exact: bool,
    all: bool,
    no_check: bool,
    files: Vec<String>,
}

impl Cmd {
    fn from_args() -> Self {
        let app = App::new("wh")
            .version(VERSION)
            .about(ABOUT)
            .author("Taylan GÖkkaya<github.com/insomnimus>")
            .usage("wh [OPTIONS] <FILE...>")
            .help_message("show this message and exit")
            .setting(AppSettings::NoBinaryName)
            .template(
                "wh, {about}
usage: 
	{usage}
	{all-args}",
            );

        let no_check = Arg::with_name("no-check")
            .long("no-check")
            .help("do not ignore patterns with only '*'");

        let f_all = Arg::with_name("all")
            .long("all")
            .short("a")
            .help("do not stop after the first match but print them all");

        let f_exact = Arg::with_name("exact").short("e").long("exact");
        #[cfg(windows)]
        let f_exact = f_exact.help("do not expand glob patterns and do not append missing '.exe'");
        #[cfg(not(windows))]
        let f_exact = f_exact.help("do not expand glob patterns");

        let f_targets = Arg::with_name("file").multiple(true).required(true);

        let matches = app
            .arg(f_all)
            .arg(f_exact)
            .arg(no_check)
            .arg(f_targets)
            .get_matches();

        let exact = matches.is_present("exact");
        let no_check = if !exact {
            matches.is_present("no-check")
        } else {
            true
        };

        let all = matches.is_present("all");

        let files: Vec<String> = matches
            .values_of("file")
            .unwrap()
            .map(|s| s.to_string())
            .skip(1)
            .collect();
        Self {
            files,
            all,
            exact,
            no_check,
        }
    }

    fn execute_exact(&self) -> i32 {
        let paths = env::var("PATH").unwrap_or_else(|e| {
            eprintln!("could not read the value of $PATH: {:?}", e);
            exit(1);
        });
        let paths: Vec<PathBuf> = env::split_paths(&paths).collect();
        let mut exit_code = 0i32;

        for t in &self.files {
            let mut found = false;
            for p in &paths {
                let mut tmp = p.clone();
                tmp.push(&t);
                if tmp.exists() {
                    found = true;
                    println!("{}", tmp.display());
                    if !self.all {
                        break;
                    }
                }
            }
            if !found {
                exit_code = 2;
                println!("{}: not found", &t);
            }
        }
        exit_code
    }

    fn execute_expand(&self) -> i32 {
        let paths = env::var("PATH").unwrap_or_else(|e| {
            eprintln!("could not read the value of $PATH: {:?}", e);
            exit(1);
        });

        #[cfg(windows)]
        let mut targets = vec![];
        #[cfg(not(windows))]
        let targets = self.files;
        #[cfg(windows)]
        for t in &self.files {
            targets.push(if is_glob(&t) || t.ends_with(".exe") {
                t.to_owned()
            } else {
                t.to_owned() + ".exe"
            });
        }

        let mut exit_code = 0i32;
        let files: Vec<PathBuf> = env::split_paths(&paths)
            .map(|p| p.read_dir())
            .filter_map(|e| e.ok())
            .flatten()
            .filter_map(|e| e.ok())
            .map(|p| p.path())
            .collect();

        const OPT: MatchOptions = MatchOptions {
            case_sensitive: false,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };

        for t in &targets {
            if !self.no_check && t == "*" {
                eprintln!("*: ignored because the --no-check flag was not set");
                continue;
            }
            let mut found = false;
            let glob = match Pattern::new(&t) {
                Ok(g) => g,
                Err(_) => {
                    exit_code = 1;
                    eprintln!("invalid glob pattern '{}'", &t);
                    continue;
                }
            };
            let g = is_glob(&t[..]);
            for f in &files {
                let s = match f.file_name() {
                    Some(n) => match n.to_str() {
                        Some(name) => name,
                        _ => continue,
                    },
                    _ => continue,
                };
                if glob.matches_with(&s[..], OPT) {
                    found = true;
                    println!("{}", f.display());
                    if !self.all && !g {
                        break;
                    }
                }
            }
            if !found {
                exit_code = 2;
                if t.contains('*') || t.contains('[') || t.contains('?') {
                    eprintln!("{}: no matches", &t);
                } else {
                    eprintln!("{}: not found", &t);
                }
            }
        }

        exit_code
    }

    fn execute(&self) -> i32 {
        if self.exact {
            self.execute_exact()
        } else {
            self.execute_expand()
        }
    }
}

fn is_glob(s: &str) -> bool {
    for &c in s.as_bytes() {
        if c == b'*' || c == b'?' || c == b'[' {
            return true;
        }
    }
    false
}

fn main() {
    let app = Cmd::from_args();
    if app.files.is_empty() {
        if app.exact || app.all || app.no_check {
            eprintln!("missing argument: file");
            exit(1);
        }
        eprintln!("wh, {}\nuse with --help for the usage", ABOUT);
        exit(0);
    }

    exit(app.execute());
}
