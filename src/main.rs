extern crate clap;
extern crate glob;

use clap::{App, AppSettings, Arg};
use glob::{MatchOptions, Pattern};
use std::env;
use std::path::PathBuf;
use std::process::exit;

const VERSION: &str = "1.0.1";
const ABOUT: &str = "find files under $PATH";

struct Cmd {
    complete: String,
    exact: bool,
    all: bool,
    no_check: bool,
    files: Vec<String>,
}

impl Cmd {
    fn app() -> App<'static> {
        let app = App::new("wh")
            .version(VERSION)
            .about(ABOUT)
            .author("Taylan GÖkkaya<github.com/insomnimus>")
            .override_usage("wh [OPTIONS] <FILE...>")
            .setting(AppSettings::NoBinaryName)
            .help_template(
                "wh, {about}
usage: 
	{usage}
	{all-args}",
            );

        let complete = Arg::new("completions")
            .long("completions")
            .about("generate shell autocompletions")
            .takes_value(true)
            .possible_values(&["bash", "elvish", "fish", "powershell", "zsh"]);

        let no_check = Arg::new("no-check")
            .long("no-check")
            .about("do not ignore patterns with only '*'")
            .conflicts_with("exact");

        let f_all = Arg::new("all")
            .long("all")
            .short('a')
            .about("do not stop after the first match but print them all");

        let f_exact = Arg::new("exact").short('e').long("exact");
        #[cfg(windows)]
        let f_exact = f_exact.about("do not expand glob patterns and do not append missing '.exe'");
        #[cfg(not(windows))]
        let f_exact = f_exact.about("do not expand glob patterns");

        let f_targets = Arg::new("file").multiple(true).required(true);
        app.arg(f_all)
            .arg(f_exact)
            .arg(no_check)
            .arg(complete)
            .arg(f_targets)
    }

    fn from_args() -> Self {
        let matches = Self::app().get_matches();

        if matches.is_present("completions") {
            return Self {
                complete: matches.value_of("completions").unwrap().to_string(),
                no_check: false,
                all: false,
                exact: false,
                files: vec![],
            };
        }
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

        #[cfg(windows)]
        let mut files = files;
        #[cfg(windows)]
        if !exact {
            for f in files.iter_mut() {
                if !f.contains('.') && !f.ends_with('*') {
                    *f += ".exe";
                }
            }
        }
        Self {
            files,
            all,
            exact,
            no_check,
            complete: String::new(),
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

        for t in &self.files {
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

    fn generate_completions(sh: &str) {
        use clap_generate::{
            generate,
            generators::{Bash, Elvish, Fish, PowerShell, Zsh},
        };
        use std::io;

        let mut app = Self::app();
        match sh {
            "bash" => generate::<Bash, _>(&mut app, "wh", &mut io::stdout()),
            "elvish" => generate::<Elvish, _>(&mut app, "wh", &mut io::stdout()),
            "fish" => generate::<Fish, _>(&mut app, "wh", &mut io::stdout()),
            "powershell" => generate::<PowerShell, _>(&mut app, "wh", &mut io::stdout()),
            "zsh" => generate::<Zsh, _>(&mut app, "wh", &mut io::stdout()),
            _ => panic!("impossible route"),
        };
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
    if !app.complete.is_empty() {
        Cmd::generate_completions(&app.complete[..]);
        return;
    }
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
