use std::collections::BTreeMap;

fn main() {
}

#[allow(dead_code)]
#[allow(clippy::needless_lifetimes)]
fn parse<'a>(_: &'a str) -> BTreeMap<&'a str, &'a str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_example() {
        let example = indoc!{r#"
            endpoint = localhost:3000
            debug = true
            log.file = /var/log/console.log
        "#};
        assert_eq!(parse(example), BTreeMap::from([
            ("endpoint", "localhost:3000"),
            ("debug", "true"),
            ("log.file", "/var/log/console.log"),
        ]));
    }
}
