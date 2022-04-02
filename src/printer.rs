use std::{fs, io, path, process};

use super::cli::Config;

fn write_entries<W: io::Write>(dir: String, writer: &mut W) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|err| {
            eprintln!("Problem reading input directory: {}", err);
            process::exit(1);
        })
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap_or_else(|err| {
            eprintln!("Problem parsing input directory: {}", err);
            process::exit(1);
        });

    paths.sort();

    for path in paths {
        let entry: path::PathBuf = path.iter().skip(1).collect(); // e.g. src/lib.rs => lib.rs
        let entry = entry.display().to_string();
        if entry.chars().next().unwrap() == '.' {
            continue; // skip hidden files
        } else if path.is_dir() {
            write!(writer, "\x1b[34;1m{}\x1b[0m  ", entry).unwrap();
        } else {
            write!(writer, "{}  ", entry).unwrap();
        }
    }
}

fn write_results<W: io::Write>(cli: Config, writer: &mut W) {
    let dir_len = cli.directories.len();
    for (i, dir) in cli.directories.into_iter().enumerate() {
        if dir_len > 1 {
            writeln!(writer, "{}:", dir).unwrap();
        }

        write_entries(dir, writer);
        writeln!(writer).unwrap();

        if i < dir_len - 1 {
            writeln!(writer).unwrap();
        }
    }
}

pub fn print(cli: Config) {
    write_results(cli, &mut io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn print_dir() {
        let want = "abc  \u{1b}[34;1mhello\u{1b}[0m  test_file  \u{1b}[34;1mtests1\u{1b}[0m  \u{1b}[34;1mtests2\u{1b}[0m  \n";

        let cli = Config::new(vec![String::from("program_name"), String::from("tests")]).unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }

    #[test]
    fn print_multiple_dir() {
        let want = "tests/tests1:\ntests1/a  tests1/b  \n\ntests/tests2:\ntests2/c  tests2/d  \n";

        let cli = Config::new(vec![String::from("program_name"), String::from("tests/tests1"), String::from("tests/tests2")]).unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }
}
