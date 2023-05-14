use std::fs;
use std::path::PathBuf;

use clap::Parser;

fn main() -> Result<(), String> {
    let file = Cli::parse().file;
    let content = fs::read_to_string(&file)
        .map_err(|err|format!("Error opening {}: {}", file.display(), err))?;
    for sysctl in parse(&content)? {
        println!("{sysctl:?}");
    }
    Ok(())
}

#[derive(Parser)]
struct Cli {
    /// sysctl.conf file to parse
    file: PathBuf,
    /// schema to validate file against
    schema: Option<PathBuf>,
}

#[derive(Debug, PartialEq)]
struct Sysctl<'a> {
    variable: &'a str,
    value: Value<'a>,
    ignore_failure: bool,
}

#[derive(Debug, PartialEq)]
enum Value<'a> {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Str(&'a str),
}

impl <'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        use Value::*;
        if let Ok(bool) = value.parse() { Bool(bool) }

        else if let Ok(u8) = value.parse() { U8(u8) }
        else if let Ok(u16) = value.parse() { U16(u16) }
        else if let Ok(u32) = value.parse() { U32(u32) }
        else if let Ok(u64) = value.parse() { U64(u64) }
        else if let Ok(u128) = value.parse() { U128(u128) }

        else if let Ok(i8) = value.parse() { I8(i8) }
        else if let Ok(i16) = value.parse() { I16(i16) }
        else if let Ok(i32) = value.parse() { I32(i32) }
        else if let Ok(i64) = value.parse() { I64(i64) }
        else if let Ok(i128) = value.parse() { I128(i128) }

        else { Str(value) }
    }
}

fn parse(input: &str) -> Result<Vec<Sysctl>, String>
{
    input.lines()
        .map(parse_line)
        .flat_map(transpose)
        .collect()
}

/// Parse a single line into variable-value pair, `Ok((variable, value))`.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing `=` or a variable.
fn parse_line(line: &str) -> Result<Option<Sysctl>, String>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c: char| c == ';' || c =='#') {
        return Ok(None)
    }

    let (line, ignore_failure) = match line.strip_prefix('-') {
        Some(line) => (line.trim(), true),  // Remove whitespace after `-`.
        None => (line, false),
    };

    let (variable, value) = line.split_once('=').ok_or("missing =")?;
    let (variable, value) = (variable.trim(), value.trim());
    if variable.is_empty() {
        return Err("missing variable".to_string())
    }
    Ok(Some(Sysctl {variable, value: value.into(), ignore_failure}))
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
    use indoc::indoc;  // To indent sample texts.

    #[test]
    fn test_example() -> Result<(), String>{
        use Value::*;
        let example = indoc!{r#"
            endpoint = localhost:3000
            debug = true
            log.file = /var/log/console.log
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { variable: "endpoint", value: Str("localhost:3000"), ignore_failure: false },
            Sysctl { variable: "debug", value: Bool(true), ignore_failure: false },
            Sysctl { variable: "log.file", value: Str("/var/log/console.log"), ignore_failure: false },
        ]));
        Ok(())
    }

    #[test]
    fn test_ignore_failure() -> Result<(), String>{
        use Value::*;
        let example = indoc!{r#"
            endpoint = localhost:3000
            -debug = true
            - log.file = /var/log/console.log
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { variable: "endpoint", value: Str("localhost:3000"), ignore_failure: false },
            Sysctl { variable: "debug", value: Bool(true), ignore_failure: true },
            Sysctl { variable: "log.file", value: Str("/var/log/console.log"), ignore_failure: true },
        ]));
        Ok(())
    }

    /// Sample from sysctl.conf(5) man page.
    #[test]
    fn test_sysctl_conf_sample() -> Result<(), String>{
        use Value::*;
        let example = indoc!{r#"
            # sysctl.conf sample
            #
            kernel.domainname = example.com
            ; this one has a space which will be written to the sysctl!
            kernel.modprobe = /sbin/mod probe
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { variable: "kernel.domainname", value: Str("example.com"), ignore_failure: false },
            Sysctl { variable: "kernel.modprobe", value: Str("/sbin/mod probe"), ignore_failure: false },
        ]));
        Ok(())
    }

    #[test]
    fn test_ints() -> Result<(), String>{
        use Value::*;
        let example = indoc!{r#"
            u8 = 0
            u16 = 1024
            i8 = -1
            i16 = -1024
        "#};
        assert_eq!(parse(example)?, Vec::from([
            Sysctl { variable: "u8", value: U8(0), ignore_failure: false },
            Sysctl { variable: "u16", value: U16(1024), ignore_failure: false },
            Sysctl { variable: "i8", value: I8(-1), ignore_failure: false },
            Sysctl { variable: "i16", value: I16(-1024), ignore_failure: false },
        ]));
        Ok(())
    }}
