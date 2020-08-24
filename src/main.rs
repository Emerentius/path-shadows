extern crate same_file;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::env;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::path;
use same_file::is_same_file;
use structopt::StructOpt;

#[derive(Clone, Copy)]
enum ShowSameFiles {
    False,
    True,
    Only,
}

use ShowSameFiles::*;
impl ShowSameFiles {
    fn show_same(&self) -> bool {
        match *self {
            False => false,
            True | Only => true,
        }
    }

    fn show_diff(&self) -> bool {
        match *self {
            False | True => true,
            Only => false,
        }
    }
}

impl std::str::FromStr for ShowSameFiles {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "false" => Ok(ShowSameFiles::False),
            "true" => Ok(ShowSameFiles::True),
            "only" => Ok(ShowSameFiles::Only),
            _      => Err("invalid option"),
        }
    }

}

#[derive(StructOpt)]
#[structopt(name = "path-shadows", about = "Find programs on PATH that are shadowed by programs in earlier directories")]
enum Opt {
    #[structopt(name = "shadows")]
    Shadows {
        /// A flag, true if used in the command line.
        #[structopt(
            short = "s",
            long = "show-same",
            help = "Show paths that point to the same file",
            // panics for some reason
            //possible_values = &["true", "false", "only"],
            raw(possible_values = r#"&["true", "false", "only"]"#),
            default_value = "false",
        )]
        show_same_file: ShowSameFiles,
        #[structopt(name = "PATH", help = "Explicit PATH to search instead of the environment variable")]
        path: Option<String>,
        #[structopt(
            short = "d",
            long = "delimiter",
            help = "Delimiter between shadowing and shadowed paths",
            default_value = ":")]
        delim: String,
    },

    #[structopt(name = "where")]
    Where {
        #[structopt(name = "COMMAND_NAME")]
        commands: Vec<std::path::PathBuf>
    },

    #[structopt(name = "validate")]
    Validate {},
}

fn main() {
    let options = Opt::from_args();

    let result = match options {
        Opt::Shadows { show_same_file, path, delim } => find_shadows(show_same_file, path, delim),
        Opt::Where { commands } => find_positions(commands),
        Opt::Validate {} => validate_path(),
    };

    if let Err(e) = result {
        eprintln!("Could not get PATH variable: {}", e);
    }
}

fn validate_path() -> Result<(), std::env::VarError> {
    let path = env::var("PATH")?;

    let mut seen_before = HashSet::new();

    for subpath_str in path.split(':') {
        if subpath_str.is_empty() {
            println!("PATH starts with or ends on ':' or contains '::' which causes it to match whatever the current directory is.");
            continue
        }

        let subpath = std::path::Path::new(subpath_str);
        // FIXME: Count duplications and only report once
        if !seen_before.insert(subpath_str) {
            println!("{} is duplicated", subpath_str);
        } else if subpath.is_relative() {
            println!("{} is not an absolute path", subpath_str)
        } else if !subpath.exists() {
            println!("{} does not exist", subpath_str);
        } else if !subpath.is_dir() {
            println!("{} is not a directory", subpath_str);
        }

    }
    Ok(())
}

fn find_positions(commands: Vec<std::path::PathBuf>) -> Result<(), std::env::VarError> {
    let mut command_found = vec![false; commands.len()];
    let path = env::var("PATH")?;

    for subpath in path.split(':') {
        let dir = match std::fs::read_dir(subpath) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Can not read directory {}: {}", subpath, e);
                continue
            }
        };

        for file in dir {
            let file = match file {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Can not read file: {}", e); // TODO: print path
                    continue
                },
            };

            // no file is stdin
            let file_type = file.file_type().expect("file type is stdin");

            if file_type.is_dir() { continue }
            let file_name = file.file_name();

            // commands can be duplicated
            for (i, _) in commands.iter().enumerate().filter(|&(_, command)| command == &file_name) {
                println!("{}", file.path().display());
                command_found[i] = true;
            }
        }
    }

    for (command, _) in commands.iter().zip(command_found).filter(|&(_, found)| !found) {
        println!("{} not found", command.display());
    }
    Ok(())
}

fn find_shadows(show_same_file: ShowSameFiles, path: Option<String>, delim: String) -> Result<(), std::env::VarError> {
    let path = path.ok_or(()).or_else(|_| env::var("PATH"))?;

    // prog -> path
    let mut duplicate_progs = HashMap::new();

    for subpath in path.split(':') {
        let dir = match std::fs::read_dir(subpath) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Can not read directory {}: {}", subpath, e);
                continue
            }
        };

        for file in dir {
            let file = match file {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Can not read file: {}", e); // TODO: print path
                    continue
                },
            };

            // no file is stdin
            let file_type = file.file_type().expect("file type is stdin");

            if file_type.is_dir() { continue }

            match duplicate_progs.entry(file.path().file_name().expect("path ends on ..").to_owned()) {
                Entry::Occupied(shadowing_path) => {
                    let shadowing_path: &path::PathBuf = shadowing_path.get();
                    match is_same_file(shadowing_path, file.path()) {
                        Ok(is_same) => {
                            if (is_same && show_same_file.show_same())
                            || (!is_same && show_same_file.show_diff()) {
                                println!("{}{}{}", file.path().display(), delim, shadowing_path.display())
                            }
                        },
                        Err(e)    => eprintln!("Can not compare {} and {}: {}", file.path().display(), shadowing_path.display(), e),
                    }
                }
                Entry::Vacant(entry) => {
                    let mut path = path::Path::new(subpath).to_path_buf();
                    path.push(file.path());
                    entry.insert(path);
                }
            };
        }

    }
    Ok(())
}
