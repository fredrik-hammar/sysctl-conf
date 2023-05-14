# `sysctl-conf`

Program to parse [sysctl.conf(5)](https://man7.org/linux/man-pages/man5/sysctl.conf.5.html)
formatted documents as an assignment for Miletos employment.

- [スキルチェック課題（1/2）](https://miletos.notion.site/1-2-c09e84f47c6743ad9ea90d9ebd3ea85e)
- [スキルチェック課題（2/2）](https://miletos.notion.site/2-2-488e7e4691e24bd48d8f200d8a43e636)

To test the program please run it with example file provided like this:

```sh
$ cargo run sysctl.conf
Sysctl { variable: "endpoint", value: "localhost:3000", ignore_failure: false }
Sysctl { variable: "debug", value: "true", ignore_failure: false }
Sysctl { variable: "log.file", value: "/var/log/console.log", ignore_failure: false }
```

Each line represents the variable to set, the value to set it to, and whether
failing to set it should be ignored.

