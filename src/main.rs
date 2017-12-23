extern crate same_file;
extern crate structopt;
#[macro_use] extern crate structopt_derive;

use std::env;
use std::collections::HashMap;
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
struct Opt {
    /// A flag, true if used in the command line.
    #[structopt(
        short = "s",
        long = "show-same",
        help = "Show paths that point to the same file",
        // panics for some reason
        //possible_values = &["true", "false", "only"],
        possible_values_raw = r#"&["true", "false", "only"]"#,
        default_value = "false",
    )]
    show_same_file: ShowSameFiles,
    #[structopt(name = "PATH", help = "Explicit PATH to search instead of the environment variable")]
    path: Option<String>
}

fn main() {
    let options = Opt::from_args();

    let path = match options.path.ok_or(())
        .or_else(|_| env::var("PATH"))
    {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Could not get PATH variable: {}", e);
            return
        },
    };

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
                            if (is_same && options.show_same_file.show_same())
                            || (!is_same && options.show_same_file.show_diff()) {
                                println!("{}:{}", file.path().display(), shadowing_path.display())
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
}
