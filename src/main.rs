extern crate same_file;

use std::env;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path;
use same_file::is_same_file;

fn main() {
    let path = match env::var("PATH") {
        Ok(p) => p,
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
                eprintln!("Can not directory {}: {}", subpath, e);
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
                        Ok(false) => println!("{}:{}", file.path().display(), shadowing_path.display()),
                        Ok(true)  => {},
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
