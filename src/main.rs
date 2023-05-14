use std::fs;

use std::path::PathBuf;
use clap::Parser;

fn main() -> Result<(), String> {
    let file = Cli::parse().file;
    let content = fs::read_to_string(&file)
        .map_err(|err|format!("Error opening {}: {}", file.display(), err))?;
    for sysctl in parse(&content)? {
        println!("{sysctl:?}", );
    }
    Ok(())
}

#[derive(Parser)]
struct Cli {
    /// sysctl.conf file to parse
    file: PathBuf,
}

#[derive(Debug, PartialEq)]
struct Sysctl<'a> {
    key: &'a str,
    value: &'a str,
    ignore_failure: bool,
}

fn parse(input: &str) -> Result<Vec<Sysctl>, String>
{
    input.lines()
        .map(parse_line)
        .flat_map(transpose)
        .collect()
}

/// Parse a single line into key-value pair, `Ok((key, value))`.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing `=` or a key.
fn parse_line(line: &str) -> Result<Option<Sysctl>, String>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c: char| c == '!' || c =='#') {
        return Ok(None)
    }

    let (line, ignore_failure) = match line.strip_prefix('-') {
        Some(line) => (line.trim(), true),  // Remove whitespace after `-`.
        None             => (line, false),
    };

    let (key, value) = line.split_once('=').ok_or("missing =")?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() {
        return Err("missing key".to_string())
    }
    Ok(Some(Sysctl {key, value, ignore_failure}))
}

/// Transposes a `Result` of an `Option` into an `Option` of a `Result`.
///
/// `Ok(None)` will be mapped to `None`.
/// `Ok(Some(_))` and `Err(_)` will be mapped to `Some(Ok(_))` and `Some(Err(_))`.
//
// Based on unstable feature for Result.
// See, https://doc.rust-lang.org/std/result/enum.Result.html#method.transpose
fn transpose<T, E>(result: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match result {
        Ok(Some(x)) => Some(Ok(x)),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_example() -> Result<(), String>{
        let example = indoc!{r#"
            endpoint = localhost:3000
            debug = true
            log.file = /var/log/console.log
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { key: "endpoint", value: "localhost:3000", ignore_failure: false },
            Sysctl { key: "debug", value: "true", ignore_failure: false },
            Sysctl { key: "log.file", value: "/var/log/console.log", ignore_failure: false },
        ]));
        Ok(())
    }

    #[test]
    fn test_ignore_failure() -> Result<(), String>{
        let example = indoc!{r#"
            endpoint = localhost:3000
            -debug = true
            - log.file = /var/log/console.log
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { key: "endpoint", value: "localhost:3000", ignore_failure: false },
            Sysctl { key: "debug", value: "true", ignore_failure: true },
            Sysctl { key: "log.file", value: "/var/log/console.log", ignore_failure: true },
        ]));
        Ok(())
    }
}
