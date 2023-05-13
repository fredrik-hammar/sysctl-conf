use std::collections::BTreeMap;

fn main() {
}

#[allow(dead_code)]
fn parse(input: &str) -> Result<BTreeMap<&str, &str>, &str>
{
    input.lines()
        .map(parse_line)
        .flat_map(transpose)
        .collect()
}

fn parse_line(line: &str) -> Result<Option<(&str, &str)>, &str>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c| c == '!' || c =='#') {
        return Ok(None)
    }
    let (key, value) = line.split_once('=').ok_or("missing =")?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() {
        return Err("missing key")
    }
    Ok(Some((key, value)))
}

/// Based on unstable feature for Result.
/// See, https://doc.rust-lang.org/std/result/enum.Result.html#method.transpose
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
    fn test_example() -> Result<(), &'static str>{
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
