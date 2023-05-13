use std::collections::BTreeMap;

fn main() {
}

#[allow(dead_code)]
fn parse(input: &str) -> Result<BTreeMap<&str, &str>, &str>
{
    input.lines()
        .flat_map(parse_line)
        .collect()
}

fn parse_line(line: &str) -> Option<Result<(&str, &str), &str>>
{
    let line = line.trim();
    // Ignore if comment.
    if line.starts_with(|c| c == '!' || c =='#') {
        return None
    }
    let split = line.split_once('=');
    if split.is_none() {
        return Some(Err("missing ="));
    }
    let (key, value) = split?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() {
        return Some(Err("missing key"))
    }
    Some(Ok((key, value)))
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
