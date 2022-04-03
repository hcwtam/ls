use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub directories: Vec<String>,
    pub flags: HashSet<char>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            directories: vec![],
            flags: HashSet::new(),
        }
    }
}

impl Config {
    pub fn new(args: Vec<String>) -> Result<Config, &'static str> {
        let mut cli = Config::default();

        // No arguments provided
        if args.len() == 1 {
            cli.directories.push(String::from("."));
        };

        let mut iter = args.into_iter();
        // skip program name
        iter.next();

        for arg in iter {
            match arg.chars().next().unwrap() {
                '-' => {
                    let flags_iter = arg.chars().enumerate();
                    for (i, flag) in flags_iter {
                        if i == 0 {
                            continue;
                        }
                        _ = cli.flags.insert(flag);
                    }
                }
                _ => cli.directories.push(arg),
            }
        }

        Ok(cli)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_empty_args() {
        let want = Config {
            directories: vec![String::from(".")],
            flags: HashSet::new(),
        };
        let get = Config::new(vec![String::from("program_name")]).unwrap();
        assert_eq!(want, get);
    }

    #[test]
    fn config_directories_args() {
        let want = Config {
            directories: vec!["src", "bin", "test"]
                .into_iter()
                .map(|a| String::from(a))
                .collect(),
            flags: HashSet::new(),
        };

        let get = Config::new(
            vec!["program_name", "src", "bin", "test"]
                .into_iter()
                .map(|a| String::from(a))
                .collect(),
        )
        .unwrap();
        assert_eq!(want, get);
    }

    #[test]
    fn config_flags_args() {
        let want = Config {
            directories: vec![],
            flags: vec!['l', 'a', 'd'].into_iter().collect::<HashSet<char>>(),
        };

        let get = Config::new(
            vec!["program_name", "-la", "-d"]
                .into_iter()
                .map(|a| String::from(a))
                .collect(),
        )
        .unwrap();
        assert_eq!(want, get);
    }

    #[test]
    fn config_mixed_args() {
        let want = Config {
            directories: vec!["src", "test"]
                .into_iter()
                .map(|a| String::from(a))
                .collect(),
            flags: vec!['l', 'a', 'd'].into_iter().collect::<HashSet<char>>(),
        };

        let get = Config::new(
            vec!["program_name", "-la", "src", "-d", "test"]
                .into_iter()
                .map(|a| String::from(a))
                .collect(),
        )
        .unwrap();
        assert_eq!(want, get);
    }
}
