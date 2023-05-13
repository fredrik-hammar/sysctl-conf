use std::collections::BTreeMap;
use std::fs;

fn main() -> Result<(), String> {
    let path = "sysctl.conf";
    let content = fs::read_to_string(path)
        .map_err(|err|format!("Error opening {path}: {err}"))?;
    println!("{:?}", parse(&content)?);
    Ok(())
}

#[allow(dead_code)]
fn parse(input: &str) -> Result<BTreeMap<&str, &str>, String>
{
    input.lines()
        .map(parse_line)
        .flat_map(transpose)
        .collect()
}

/// Parse a single line into key-value pair, `Ok((key, value))`.
/// Empty or comment lines will result in `Ok(None)`.
/// Error if line is missing `=` or a key.
fn parse_line(line: &str) -> Result<Option<(&str, &str)>, String>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c| c == '!' || c =='#') {
        return Ok(None)
    }
    let (key, value) = line.split_once('=').ok_or("missing =")?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() {
        return Err("missing key".to_string())
    }
    Ok(Some((key, value)))
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
        assert_eq!(parse(example)?, BTreeMap::from([
            ("endpoint", "localhost:3000"),
            ("debug", "true"),
            ("log.file", "/var/log/console.log"),
        ]));
        Ok(())
    }
}
