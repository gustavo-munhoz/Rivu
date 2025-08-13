#[inline]
pub fn strip_surrounding_quotes(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2 {
        let first = b[0];
        let last = b[b.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

pub fn split_csv_preserving_quotes(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes: Option<char> = None;

    for ch in line.chars() {
        match in_quotes {
            Some(q) => {
                if ch == q {
                    in_quotes = None;
                    cur.push(ch);
                } else {
                    cur.push(ch);
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    in_quotes = Some(ch);
                    cur.push(ch);
                } else if ch == ',' {
                    out.push(cur.trim().to_string());
                    cur.clear();
                } else {
                    cur.push(ch);
                }
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_quotes_works() {
        assert_eq!(strip_surrounding_quotes("'a,b'"), "a,b");
        assert_eq!(strip_surrounding_quotes(r#""x""#), "x");
        assert_eq!(strip_surrounding_quotes("nq"), "nq");
    }

    #[test]
    fn split_preserving_quotes() {
        let line = r#"'sunny',85,"85",FALSE,no"#;
        let p = split_csv_preserving_quotes(line);
        assert_eq!(p, vec!["'sunny'", "85", "\"85\"", "FALSE", "no"]);
    }
}
