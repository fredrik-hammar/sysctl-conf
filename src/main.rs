use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

fn main() -> Result<(), String> {
    let Cli {file, schema } = Cli::parse();
    let content = fs::read_to_string(&file)
        .map_err(|err|format!("Error opening {}: {}", file.display(), err))?;
    let sysctls = parse_sysctl(&content)?;
    for sysctl in &sysctls {
        println!("{sysctl:?}");
    }
    if let Some(schema) = schema {
        let schema = fs::read_to_string(schema)
            .map_err(|err|format!("Error opening {}: {}", file.display(), err))?;
        let schema = Schema::parse(&schema)?;
        schema.validate(sysctls)?;
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

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(value) => write!(f, "{value}"),
            Value::U8(value) => write!(f, "{value}"),
            Value::U16(value) => write!(f, "{value}"),
            Value::U32(value) => write!(f, "{value}"),
            Value::U64(value) => write!(f, "{value}"),
            Value::U128(value) => write!(f, "{value}"),
            Value::I8(value) => write!(f, "{value}"),
            Value::I16(value) => write!(f, "{value}"),
            Value::I32(value) => write!(f, "{value}"),
            Value::I64(value) => write!(f, "{value}"),
            Value::I128(value) => write!(f, "{value}"),
            Value::Str(value) => write!(f, "{value}"),
        }
    }
}

fn parse_sysctl(input: &str) -> Result<Vec<Sysctl>, String>
{
    input.lines()
        .map(parse_sysctl_line)
        .flat_map(transpose)
        .collect()
}

/// Parse a single line into variable-value pair, `Ok((variable, value))`.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing `=` or a variable.
fn parse_sysctl_line(line: &str) -> Result<Option<Sysctl>, String>
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

#[derive(Debug, PartialEq)]
struct Schema<'a> {
    map: BTreeMap<&'a str, Type>
}

impl Schema<'_> {
    fn parse(content: &str) -> Result<Schema, String> {
        Ok(Schema {
            map: content.lines()
                .map(parse_schema_line)
                .flat_map(transpose)
                .map(|e| e.map(SchemaEntry::into))
                .collect::<Result<BTreeMap<&str, Type>, String>>()?
        })
    }

    fn validate(&self, sysctls: Vec<Sysctl>) -> Result<(), String> {
        for sysctl in sysctls {
            let Sysctl { variable, value, ignore_failure } = sysctl;
            if ignore_failure { continue }
            let r#type = self.map.get(variable).ok_or(format!("not in schema: {variable}"))?;
            let value = value.to_string();
            r#type.validate(&value)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
struct SchemaEntry<'a> {
    variable: &'a str,
    r#type: Type,
}

impl <'a> From<SchemaEntry<'a>> for (&'a str, Type) {
    fn from(value: SchemaEntry<'a>) -> Self {
        (value.variable, value.r#type)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Str,
}

impl Type {
    fn validate(self, value: &str) -> Result<(), String> {
        match self {
            Type::Bool => self.validate_helper(bool::from_str, value),
            Type::U8 => self.validate_helper(u8::from_str, value),
            Type::U16 => self.validate_helper(u16::from_str, value),
            Type::U32 => self.validate_helper(u32::from_str, value),
            Type::U64 => self.validate_helper(u64::from_str, value),
            Type::U128 => self.validate_helper(u128::from_str, value),
            Type::I8 => self.validate_helper(i8::from_str, value),
            Type::I16 => self.validate_helper(i16::from_str, value),
            Type::I32 => self.validate_helper(i32::from_str, value),
            Type::I64 => self.validate_helper(i64::from_str, value),
            Type::I128 => self.validate_helper(i128::from_str, value),
            Type::Str => Ok(()),
        }
    }

    fn validate_helper<F, T, E>(self, from_str: F, value: &str) -> Result<(), String>
    where F: FnOnce(&str) -> Result<T, E> {
        from_str(value)
            .map(|_| ())
            .map_err(|_| {
                format!("not of type {self:?}: {value}")
            })
    }
}

impl FromStr for Type {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Type::*;
        match s {
            "bool" => Ok(Bool),
            "u8" => Ok(U8),
            "u16" => Ok(U16),
            "u32" => Ok(U32),
            "u64" => Ok(U64),
            "u128" => Ok(U128),
            "i8" => Ok(I8),
            "i16" => Ok(I16),
            "i32" => Ok(I32),
            "i64" => Ok(I64),
            "i128" => Ok(I128),
            "string" => Ok(Str),
            _ => Err(format!("{s} is not a type"))
        }
    }
}


/// Parse a single line into variable-type pair.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing `:` or a variable.
fn parse_schema_line(line: &str) -> Result<Option<SchemaEntry>, String>
{
    if let Some((variable, r#type)) = parse_line_pair(line, ":")? {
        Ok(Some(SchemaEntry { variable, r#type: r#type.parse()? }))
    } else {
        Ok(None)
    }
}

/// Parse a single line into variable-value pair, `Ok((variable, value))`.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing delimiter or a variable.
fn parse_line_pair<'a>(line: &'a str, delimiter: &str) -> Result<Option<(&'a str, &'a str)>, String>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c: char| c == ';' || c =='#') {
        return Ok(None)
    }

    let (variable, value) = line.split_once(delimiter).ok_or("missing =")?;
    let (variable, value) = (variable.trim(), value.trim());
    if variable.is_empty() {
        return Err("missing variable".to_string())
    }
    Ok(Some((variable, value)))
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
        assert_eq!(parse_sysctl(example)?, Vec::from([
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
        assert_eq!(parse_sysctl(example)?, Vec::from([
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
        assert_eq!(parse_sysctl(example)?, Vec::from([
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
        assert_eq!(parse_sysctl(example)?, Vec::from([
            Sysctl { variable: "u8", value: U8(0), ignore_failure: false },
            Sysctl { variable: "u16", value: U16(1024), ignore_failure: false },
            Sysctl { variable: "i8", value: I8(-1), ignore_failure: false },
            Sysctl { variable: "i16", value: I16(-1024), ignore_failure: false },
        ]));
        Ok(())
    }}
