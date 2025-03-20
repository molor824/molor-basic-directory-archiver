use std::{
    env, fs,
    path::{self, PathBuf},
};

mod archive;
mod extract;

fn main() {
    let mut args = env::args().skip(1);
    let mut path = None;
    let mut output_path = None;
    let mut extract = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output_path = match args.next() {
                    None => {
                        eprintln!("Expected output path");
                        return;
                    }
                    p => p,
                };
            }
            "-x" | "--extract" => extract = true,
            _ => {
                if path.is_none() {
                    path = Some(arg);
                    continue;
                }
                break;
            }
        }
    }

    let Some(path) = path.map(|p| path::absolute(p).unwrap()) else {
        eprintln!("Expected path");
        return;
    };

    if extract {
        let buffer = fs::read(&path).unwrap();
        let output_path = match output_path {
            Some(p) => PathBuf::from(p),
            None => match path.parent() {
                Some(p) => p.to_path_buf(),
                None => {
                    eprintln!(
                        "Cannot extract to the parent directory of input file. Try explicitly specifying the output directory with `-o`"
                    );
                    return;
                }
            },
        };

        extract::from_buffer(&buffer, output_path).unwrap();
    } else {
        let output_path = match output_path {
            Some(p) => PathBuf::from(p),
            None => {
                let mut clone = path.clone();
                clone.set_extension("mbds");
                clone
            }
        };

        let mut buffer = vec![];
        archive::to_buffer(path, &mut buffer).unwrap();
        fs::write(output_path, buffer).unwrap();
    }
}
