//! A code span after an unmatched longer backtick string (cmark PR #632): the closer cache must
//! not move back to an earlier position, or a later opener of that length misses its closer.

use super::*;
use pretty_assertions::assert_eq;

/// Expected outputs taken from commonmark.js 0.31.2.
#[test]
fn code_span_after_unmatched_longer_backtick_string() {
    html("`` `a` `b`\n", "<p>`` <code>a</code> <code>b</code></p>\n");
    html(
        "a `b` c`` d `e` f `g` h\n",
        "<p>a <code>b</code> c`` d <code>e</code> f <code>g</code> h</p>\n",
    );
    html(
        "``` `a` `` `b` ``\n",
        "<p>``` <code>a</code> <code>`b`</code></p>\n",
    );
    html(
        "`a` `` `b` `c`\n",
        "<p><code>a</code> `` <code>b</code> <code>c</code></p>\n",
    );
}

/// CommonMark 6.1 read literally: an opener closes at the next backtick string of its exact
/// length, else it is literal text. Checked against commonmark.js 0.31.2 on every string of
/// length 1 to 9 over this alphabet.
fn code_spans_by_spec(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let run = |mut i: usize| {
        let start = i;
        while i < b.len() && b[i] == b'`' {
            i += 1;
        }
        i - start
    };
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let len = run(i);
        let mut k = i + len;
        let mut closer = None;
        while k < b.len() {
            if b[k] != b'`' {
                k += 1;
                continue;
            }
            let n = run(k);
            if n == len {
                closer = Some(k);
                break;
            }
            k += n;
        }
        match closer {
            None => i += len,
            Some(c) => {
                let mut code = &s[i + len..c];
                if code.len() >= 2
                    && code.starts_with(' ')
                    && code.ends_with(' ')
                    && code.bytes().any(|x| x != b' ')
                {
                    code = &code[1..code.len() - 1];
                }
                out.push(code.to_string());
                i = c + len;
            }
        }
    }
    out
}

fn code_spans_by_comrak(s: &str) -> Vec<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, s, &Options::default());
    root.descendants()
        .filter_map(|n| match &n.data().value {
            NodeValue::Code(c) => Some(c.literal.to_string()),
            _ => None,
        })
        .collect()
}

/// Every paragraph of 1 to 10 bytes over `` ` ``, `a` and space (no leading or trailing space,
/// which the paragraph strips): comrak's code spans equal the spec's.
#[test]
fn code_spans_match_the_spec_exhaustively() {
    let alphabet = b"`a ";
    let mut checked = 0;
    for len in 1..=10u32 {
        for mut n in 0..3usize.pow(len) {
            let mut s = Vec::with_capacity(len as usize);
            for _ in 0..len {
                s.push(alphabet[n % 3]);
                n /= 3;
            }
            if s[0] == b' ' || s[s.len() - 1] == b' ' {
                continue;
            }
            let s = String::from_utf8(s).unwrap();
            assert_eq!(
                code_spans_by_comrak(&s),
                code_spans_by_spec(&s),
                "input {s:?}"
            );
            checked += 1;
        }
    }
    assert!(checked > 30_000);
}
