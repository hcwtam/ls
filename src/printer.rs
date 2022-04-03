use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{collections::HashSet, fs, io, process};

use chrono::{TimeZone, Utc};
use users;

use super::cli::Config;

const KILO: f64 = 1000.0;
const MEGA: f64 = KILO * 1000.0;
const GIGA: f64 = MEGA * 1000.0;
const TERA: f64 = GIGA * 1000.0;

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
        write_dir_entry(".", flags.contains(&'F'), writer);
        write_dir_entry("..", flags.contains(&'F'), writer);
    }

    for path in paths {
        let entry = path.strip_prefix(&dir).unwrap().display().to_string(); // e.g. src/lib.rs => lib.rs
        if entry.chars().next().unwrap() == '.' && !flags.contains(&'a') {
            continue; // skip hidden files if -a is not enabled
        } else if path.is_dir() {
            write_dir_entry(&entry, flags.contains(&'F'), writer);
        } else {
            write!(writer, "{}  ", entry).unwrap();
        }
    }
}

// When -l is enabled
fn write_long_entries<W: io::Write>(dir: String, writer: &mut W, flags: &HashSet<char>) {
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
        write_metadata(".", writer);
        write_dir_entry(".", flags.contains(&'F'), writer);
        writeln!(writer).unwrap();

        write_metadata("..", writer);
        write_dir_entry("..", flags.contains(&'F'), writer);
        writeln!(writer).unwrap();
    }

    for path in paths {
        let entry = path.strip_prefix(&dir).unwrap().display().to_string(); // e.g. src/lib.rs => lib.rs
        if entry.chars().next().unwrap() == '.' && !flags.contains(&'a') {
            continue; // skip hidden files if -a is not enabled
        } else if path.is_dir() {
            write_metadata(&path.display().to_string(), writer);
            write_dir_entry(&entry, flags.contains(&'F'), writer);
            writeln!(writer).unwrap();
        } else {
            write_metadata(&path.display().to_string(), writer);
            writeln!(writer, "{}", entry).unwrap();
        }
    }
}

// append "/" to directory if "-F" enabled
fn write_dir_entry<W: io::Write>(entry: &str, enabled: bool, writer: &mut W) {
    if enabled {
        write!(writer, "\x1b[34;1m{}/\x1b[0m  ", entry).unwrap();
    } else {
        write!(writer, "\x1b[34;1m{}\x1b[0m  ", entry).unwrap();
    }
}

fn write_metadata<W: io::Write>(path: &str, writer: &mut W) {
    // Read and get metadata
    let metadata = fs::metadata(path).unwrap();

    // Permissions
    let modes = metadata.permissions().mode();

    let is_dir = if metadata.is_dir() { 'd' } else { '-' };
    let index = if metadata.is_dir() { 6 } else { 7 };
    let permissions = &format!("{:b}", modes)[index..];

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

    // owner of the file
    let owner_uid = metadata.uid();
    let owner = users::get_user_by_uid(owner_uid).unwrap();
    let owner_name = owner.name().to_string_lossy();

    // file size
    let file_size = metadata.len() as f64;
    let size = if file_size >= TERA {
        format!("{:.3}T", file_size / TERA)
    } else if file_size >= GIGA {
        format!("{:.3}G", file_size / GIGA)
    } else if file_size >= MEGA {
        format!("{:.3}M", file_size / MEGA)
    } else if file_size >= KILO {
        format!("{:.3}k", file_size / KILO)
    } else {
        if file_size > 100.0 {
            format!("  {} ", file_size)
        } else if file_size > 10.0 {
            format!("   {} ", file_size)
        } else {
            format!("    {} ", file_size)
        }
    };

    // modification date and time
    let timestamp = metadata.mtime();
    let modification_time = Utc
        .timestamp(timestamp, 0)
        .format("%Y %b %e %T")
        .to_string();

    write!(
        writer,
        "\u{1b}[35;1m{}\u{1b}[0m \
    \u{1b}[33;1m{}\u{1b}[0m \
    \u{1b}[32;1m{}\u{1b}[0m \
    \u{1b}[36;1m{}\u{1b}[0m ",
        permissions_output, owner_name, size, modification_time
    )
    .unwrap();
}

fn write_results<W: io::Write>(cli: Config, writer: &mut W) {
    let dir_len = cli.directories.len();

    for (i, dir) in cli.directories.into_iter().enumerate() {
        if dir_len > 1 {
            writeln!(writer, "{}:", dir).unwrap();
        }

        if cli.flags.contains(&'l') {
            write_long_entries(dir, writer, &cli.flags);
        } else {
            write_entries(dir, writer, &cli.flags);
        }
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

    #[test]
    fn print_with_l_flag() {
        let want = "tests/tests1:\n\u{1b}[35;1m-rw-rw-r--\u{1b}[0m \u{1b}[33;1mw\u{1b}[0m \u{1b}[32;1m    0 \u{1b}[0m \u{1b}[36;1m2022 Apr  2 13:34:42\u{1b}[0m a\n\u{1b}[35;1m-rw-rw-r--\u{1b}[0m \u{1b}[33;1mw\u{1b}[0m \u{1b}[32;1m    0 \u{1b}[0m \u{1b}[36;1m2022 Apr  2 13:34:46\u{1b}[0m b\n\n\ntests/tests2:\n\u{1b}[35;1m-rw-rw-r--\u{1b}[0m \u{1b}[33;1mw\u{1b}[0m \u{1b}[32;1m    0 \u{1b}[0m \u{1b}[36;1m2022 Apr  2 13:34:52\u{1b}[0m c\n\u{1b}[35;1m-rw-rw-r--\u{1b}[0m \u{1b}[33;1mw\u{1b}[0m \u{1b}[32;1m    0 \u{1b}[0m \u{1b}[36;1m2022 Apr  2 13:34:54\u{1b}[0m d\n\n";

        let cli = Config::new(vec![
            String::from("program_name"),
            String::from("tests/tests1"),
            String::from("tests/tests2"),
            String::from("-l"),
        ])
        .unwrap();
        let mut stdout = vec![];

        write_results(cli, &mut stdout);

        assert_eq!(want, str::from_utf8(&stdout).unwrap());
    }
}
