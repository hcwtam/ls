use std::os::unix::fs::PermissionsExt;
use std::{collections::HashSet, fs, io, process};

use super::cli::Config;

fn write_entries<W: io::Write>(dir: String, writer: &mut W, flags: &HashSet<char>) {
    let mut paths = fs::read_dir(&dir)
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

    if flags.contains(&'a') {
        write_dir_entry(String::from("."), flags.contains(&'F'), writer);
        write_dir_entry(String::from(".."), flags.contains(&'F'), writer);
    }

    for path in paths {
        let entry = path.strip_prefix(&dir).unwrap().display().to_string(); // e.g. src/lib.rs => lib.rs
        if entry.chars().next().unwrap() == '.' && !flags.contains(&'a') {
            continue; // skip hidden files if -a is not enabled
        } else if path.is_dir() {
            write_dir_entry(entry, flags.contains(&'F'), writer);
        } else {
            write!(writer, "{}  ", entry).unwrap();
        }
    }
}

// append "/" to directory if "-F" enabled
fn write_dir_entry<W: io::Write>(entry: String, enabled: bool, writer: &mut W) {
    if enabled {
        write!(writer, "\x1b[34;1m{}/\x1b[0m  ", entry).unwrap();
    } else {
        write!(writer, "\x1b[34;1m{}\x1b[0m  ", entry).unwrap();
    }
}

fn write_metadata<W: io::Write>(path: &str, writer: &mut W) {
    // Read and get metadata
    let metadata = fs::metadata("the_egg.txt").unwrap();

    // Permissions
    let modes = metadata.permissions().mode();

    let permissions = &format!("{:b}", modes)[7..];
    let is_dir = if metadata.is_dir() { 'd' } else { '-' };

    let mut permissions_output = String::from(is_dir);
    for (i, bit) in permissions.chars().enumerate() {
        let mut symbol = '-';
        if bit == '1' {
            symbol = match i % 3 {
                0 => 'r',
                1 => 'w',
                2 => 'x',
                _ => unreachable!(),
            }
        }
        permissions_output.push(symbol);
    }

    // 1 : number of linked hard-links

    // lilo: owner of the file

    // lilo: to which group this file belongs to

    // 0: size

    // Feb 26 07:08 modification/creation date and time
}

fn write_results<W: io::Write>(cli: Config, writer: &mut W) {
    let dir_len = cli.directories.len();

    for (i, dir) in cli.directories.into_iter().enumerate() {
        if dir_len > 1 {
            writeln!(writer, "{}:", dir).unwrap();
        }

        write_entries(dir, writer, &cli.flags);
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
        let want = "tests/tests1:\na  b  \n\ntests/tests2:\nc  d  \n";

        let cli = Config::new(vec![
            String::from("program_name"),
            String::from("tests/tests1"),
            String::from("tests/tests2"),
        ])
        .unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }

    #[test]
    fn print_with_a_flag() {
        let want = "\u{1b}[34;1m.\u{1b}[0m  \u{1b}[34;1m..\u{1b}[0m  .bye  hi  \n";

        let cli = Config::new(vec![
            String::from("program_name"),
            String::from("tests/hello"),
            String::from("-a"),
        ])
        .unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }

    #[test]
    fn print_with_uppercase_f_flag() {
        let want = "abc  \u{1b}[34;1mhello/\u{1b}[0m  test_file  \u{1b}[34;1mtests1/\u{1b}[0m  \u{1b}[34;1mtests2/\u{1b}[0m  \n";

        let cli = Config::new(vec![
            String::from("program_name"),
            String::from("tests"),
            String::from("-F"),
        ])
        .unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }
}
