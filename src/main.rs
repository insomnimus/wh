mod app;

use glob::{MatchOptions, Pattern};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;
use walkdir::{DirEntry, WalkDir};

struct Cmd {
    recursive: bool,
    hidden: bool,
    exact: bool,
    all: bool,
    no_check: bool,
    find_under: Option<Vec<String>>,
    args: Vec<String>,
}

impl Cmd {
    fn from_args() -> Self {
        let matches = app::new().get_matches();

        let recursive = matches.is_present("recursive");
        let hidden = matches.is_present("hidden");
        let exact = matches.is_present("exact");
        let no_check = matches.is_present("no-check");
        let all = if !matches.is_present("all") {
            false
        } else {
            match matches.value_of("all") {
                None => true,
                Some(s) => s == "true" || s == "yes",
            }
        };

        let args: Vec<String> = matches
            .values_of("target")
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        let find_under = match matches.values_of("find-under") {
            None => None,
            Some(itr) => Some(itr.map(|s| s.to_string()).collect()),
        };

        #[cfg(windows)]
        let mut args = args;
        #[cfg(windows)]
        if !exact {
            for s in args.iter_mut() {
                if !is_glob(&s) && !s.ends_with(".exe") {
                    *s += ".exe";
                }
            }
        }

        Self {
            recursive,
            hidden,
            exact,
            all,
            no_check,
            find_under,
            args,
        }
    }

    fn execute_exact(&self) -> i32 {
        let paths = env::var("PATH").unwrap_or_else(|e| {
            eprintln!("could not read the value of $PATH: {:?}", e);
            exit(1);
        });

        let paths: Vec<PathBuf> = env::split_paths(&paths).collect();

        let mut exit_code = 0i32;

        for t in &self.args {
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
                exit_code = 3;
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

        for t in &self.args {
            if !self.no_check && t == "*" {
                eprintln!("*: ignored because the --no-check flag was not set");
                continue;
            }

            let mut found = false;
            let glob = match Pattern::new(&t) {
                Ok(g) => g,
                Err(_) => {
                    exit_code = 2;
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
                exit_code = 3;

                eprintln!("{}: not found", &t);
            }
        }

        exit_code
    }

    fn execute(&self) -> i32 {
        if self.find_under.is_none() {
            if self.exact {
                self.execute_exact()
            } else {
                self.execute_expand()
            }
        } else if self.recursive {
            self.execute_recursive()
        } else {
            self.find_under()
        }
    }

    fn find_under_exact(&self, paths: &Vec<impl AsRef<Path>>) -> i32 {
        let mut map: HashMap<&String, bool> = HashMap::new();

        for s in &self.args {
            map.insert(&s, false);
        }

        for p in paths {
            let walker = WalkDir::new(p.as_ref()).into_iter();
            for e in walker
                .filter_entry(|e| self.hidden || !is_hidden_dir(e))
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if !self.all && map.values().all(|&b| b && true) {
                    return 0;
                }
                let fname = match e.file_name().to_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                for s in &self.args {
                    if !self.all && *map.get(s).unwrap() {
                        continue;
                    }
                    if fname == *s {
                        map.insert(s, true);
                        println!("{}", e.path().display());
                    }
                }
            }
        }

        let mut exit_code = 0i32;
        for (k, v) in map {
            if !v {
                println!("{}: not found", k);
                exit_code = 2;
            }
        }
        exit_code
    }

    fn find_under_expand(&self, paths: &Vec<impl AsRef<Path>>) -> i32 {
        const OPT: MatchOptions = MatchOptions {
            case_sensitive: false,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };

        struct Target {
            found: bool,
            is_glob: bool,
            glob: Pattern,
        }

        let mut map: HashMap<&String, Target> = HashMap::new();

        for s in &self.args {
            if !self.no_check && s == "*" {
                println!("'*': ignored because the --no-check flag is not set");
                continue;
            }
            map.insert(
                &s,
                Target {
                    found: false,
                    is_glob: is_glob(&s[..]),
                    glob: Pattern::new(&s).unwrap_or_else(|e| {
                        eprintln!("{}: invalid glob pattern: {:?}", &s, e);
                        exit(2);
                    }),
                },
            );
        }

        if map.is_empty() {
            return 0;
        }

        for p in paths {
            // TODO: make this concurrent
            let walker = WalkDir::new(&p).into_iter();
            for e in walker
                .filter_entry(|e| self.hidden || !is_hidden_dir(e))
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if !self.all && map.iter().all(|(_, v)| v.found && !v.is_glob) {
                    return 0;
                }
                let fname = match e.file_name().to_str() {
                    Some(s) => s,
                    None => continue,
                };

                for (_, t) in map.iter_mut() {
                    if !self.all && t.found && !t.is_glob {
                        continue;
                    }
                    if t.glob.matches_with(fname, OPT) {
                        println!("{}", e.path().display());
                        t.found = true;
                    }
                }
            }
        }

        let mut exit_code = 0i32;
        for (s, t) in map {
            if !t.found {
                println!("{}: not found", s);
                exit_code = 3;
            }
        }
        exit_code
    }

    fn find_under(&self) -> i32 {
        let paths = self.find_under.as_ref().unwrap();

        if self.exact {
            self.find_under_exact(paths)
        } else {
            self.find_under_expand(paths)
        }
    }

    fn execute_recursive(&self) -> i32 {
        let paths = env::var("PATH").unwrap_or_else(|e| {
            eprintln!("could not read the value of $PATH: {:?}", e);
            exit(1);
        });

        let paths: Vec<PathBuf> = env::split_paths(&paths).collect();
        if self.exact {
            self.find_under_exact(&paths)
        } else {
            self.find_under_expand(&paths)
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

fn is_hidden_dir(e: &DirEntry) -> bool {
    if e.file_type().is_file() {
        return false;
    }
    e.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn main() {
    exit(Cmd::from_args().execute());
}
