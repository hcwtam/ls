use std::{env, error, fs, io};

fn main() -> Result<(), Box<dyn error::Error>> {
    let arguments: Vec<String> = env::args().collect();

    if arguments.len() == 1 {
        let mut paths = fs::read_dir(".")?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;

        paths.sort();

        for path in paths {
            let entry = &path.display().to_string()[2..];
            if entry.chars().next().unwrap() == '.' {
                continue;
            } else if path.is_dir() {
                print!("\x1b[34;1m{}\x1b[0m  ", entry);
            } else {
                print!("{}  ", entry);
            }
        }
        println!();
        return Ok(());
    }

    for argument in arguments {
        println!("{}", argument)
    }

    Ok(())
}
