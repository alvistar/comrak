//! `parse.leave_link_references`: link reference definitions stay in the tree as
//! `LinkDefinition` nodes, and reference-form links and images (full, collapsed, shortcut) arrive
//! as `LinkReference` and `ImageReference` nodes carrying their label and kind. With the option
//! off, every output is what upstream comrak gives.

use super::*;
use crate::nodes::{LinkReferenceKind, NodeLinkDefinition, NodeLinkReference};
use pretty_assertions::assert_eq;

fn options(leave_link_references: bool) -> Options<'static> {
    let mut o = Options::default();
    o.extension.table = true;
    o.extension.strikethrough = true;
    o.extension.tasklist = true;
    o.extension.autolink = true;
    o.extension.footnotes = true;
    o.parse.leave_link_references = leave_link_references;
    o
}

fn kind(k: LinkReferenceKind) -> &'static str {
    match k {
        LinkReferenceKind::Full => "full",
        LinkReferenceKind::Collapsed => "collapsed",
        LinkReferenceKind::Shortcut => "shortcut",
    }
}

/// One line per node in preorder: depth, name, sourcepos, and the fields this option is about.
fn dump(md: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, md, &options(true));
    let mut out = String::new();
    for edge in root.traverse() {
        let crate::arena_tree::NodeEdge::Start(node) = edge else {
            continue;
        };
        let depth = node.ancestors().count() - 1;
        let ast = node.data();
        let detail = match &ast.value {
            NodeValue::Text(t) => format!(" {t:?}"),
            NodeValue::Link(l) | NodeValue::Image(l) => format!(" {:?} {:?}", l.url, l.title),
            NodeValue::LinkDefinition(d) => {
                let NodeLinkDefinition { label, url, title } = &**d;
                format!(" [{label}] {url:?} {title:?}")
            }
            NodeValue::LinkReference(r) | NodeValue::ImageReference(r) => {
                let NodeLinkReference {
                    label,
                    kind: k,
                    url,
                    title,
                } = &**r;
                format!(" {} [{label}] {url:?} {title:?}", kind(*k))
            }
            _ => String::new(),
        };
        out.push_str(&format!(
            "{}{} ({}){}\n",
            "  ".repeat(depth),
            ast.value.xml_node_name(),
            ast.sourcepos,
            detail
        ));
    }
    out
}

#[test]
fn definitions_stay_as_nodes() {
    assert_eq!(
        dump(concat!(
            "[Foo Bar]: /url \"Title\"\n",
            "[b]:\n",
            "  </two words>\n",
            "  'T'   \n",
            "[foo bar]: /duplicate\n",
            "after\n",
            "\n",
            "> [q]: /q\n",
            "\n",
            "[s]: /s\n",
            "Heading\n",
            "===\n",
        )),
        concat!(
            "document (1:1-12:3)\n",
            "  link_definition (1:1-1:23) [Foo Bar] \"/url\" \"Title\"\n",
            "  link_definition (2:1-4:5) [b] \"/two words\" \"T\"\n",
            "  link_definition (5:1-5:21) [foo bar] \"/duplicate\" \"\"\n",
            "  paragraph (6:1-6:5)\n",
            "    text (6:1-6:5) \"after\"\n",
            "  block_quote (8:1-8:9)\n",
            "    link_definition (8:3-8:9) [q] \"/q\" \"\"\n",
            "  link_definition (10:1-10:7) [s] \"/s\" \"\"\n",
            "  heading (11:1-12:3)\n",
            "    text (11:1-11:7) \"Heading\"\n",
        )
    );
}

#[test]
fn full_collapsed_and_shortcut_links_are_reference_nodes() {
    assert_eq!(
        dump(concat!(
            "[Foo Bar]: /url \"Title\"\n",
            "[foo bar]: /duplicate\n",
            "\n",
            "A [*full*][foo  bar], [Foo Bar][] and [FOO BAR].\n",
        )),
        concat!(
            "document (1:1-4:48)\n",
            "  link_definition (1:1-1:23) [Foo Bar] \"/url\" \"Title\"\n",
            "  link_definition (2:1-2:21) [foo bar] \"/duplicate\" \"\"\n",
            "  paragraph (4:1-4:48)\n",
            "    text (4:1-4:2) \"A \"\n",
            "    link_reference (4:3-4:20) full [foo  bar] \"/url\" \"Title\"\n",
            "      emph (4:4-4:9)\n",
            "        text (4:5-4:8) \"full\"\n",
            "    text (4:21-4:22) \", \"\n",
            "    link_reference (4:23-4:33) collapsed [Foo Bar] \"/url\" \"Title\"\n",
            "      text (4:24-4:30) \"Foo Bar\"\n",
            "    text (4:34-4:38) \" and \"\n",
            "    link_reference (4:39-4:47) shortcut [FOO BAR] \"/url\" \"Title\"\n",
            "      text (4:40-4:46) \"FOO BAR\"\n",
            "    text (4:48-4:48) \".\"\n",
        )
    );
}

#[test]
fn images_are_reference_nodes() {
    assert_eq!(
        dump(concat!(
            "![alt][i] ![i][] ![i]\n",
            "\n",
            "[i]: /i.png 'T'\n",
        )),
        concat!(
            "document (1:1-3:15)\n",
            "  paragraph (1:1-1:21)\n",
            "    image_reference (1:1-1:9) full [i] \"/i.png\" \"T\"\n",
            "      text (1:3-1:5) \"alt\"\n",
            "    text (1:10-1:10) \" \"\n",
            "    image_reference (1:11-1:16) collapsed [i] \"/i.png\" \"T\"\n",
            "      text (1:13-1:13) \"i\"\n",
            "    text (1:17-1:17) \" \"\n",
            "    image_reference (1:18-1:21) shortcut [i] \"/i.png\" \"T\"\n",
            "      text (1:20-1:20) \"i\"\n",
            "  link_definition (3:1-3:15) [i] \"/i.png\" \"T\"\n",
        )
    );
}

#[test]
fn unresolved_references_stay_text_and_inline_links_stay_links() {
    assert_eq!(
        dump("[x][nope] [nope][] [nope] [y](/inline)\n"),
        concat!(
            "document (1:1-1:38)\n",
            "  paragraph (1:1-1:38)\n",
            "    text (1:1-1:26) \"[x][nope] [nope][] [nope] \"\n",
            "    link (1:27-1:38) \"/inline\" \"\"\n",
            "      text (1:28-1:28) \"y\"\n",
        )
    );
}

/// Option on: definitions render nothing and references render as the links they resolve to,
/// so the HTML is the option-off HTML.
#[test]
fn html_is_unchanged_by_the_option() {
    let input = include_str!("fixtures/link_references.md");
    let mut on = options(true);
    on.render.sourcepos = true;
    assert_eq!(
        markdown_to_html(input, &on),
        include_str!("fixtures/link_references.html")
    );
}

/// Option on: the CommonMark output writes references and definitions back in source form, and
/// that output parses to the same HTML.
#[test]
fn commonmark_output_keeps_the_source_form() {
    let md = "[Foo]: /u \"T\"\n\nA [x][foo], [Foo][], [foo] and ![i][foo].\n";
    let cm = markdown_to_commonmark(md, &options(true));
    assert_eq!(
        cm,
        "[Foo]: /u \"T\"\n\nA [x][foo], [Foo][], [foo] and ![i][foo].\n"
    );
    assert_eq!(
        markdown_to_html(&cm, &options(false)),
        markdown_to_html(md, &options(false))
    );
}

/// Option off: HTML, XML (with every sourcepos) and CommonMark output of a corpus of reference
/// shapes are byte for byte what upstream comrak 1.0.0 (with the backtick fix) produced; the
/// fixtures were written by that build before this option existed.
#[test]
fn option_off_is_byte_identical_to_upstream() {
    let input = include_str!("fixtures/link_references.md");
    let mut off = options(false);
    off.render.sourcepos = true;
    let arena = Arena::new();
    let root = parse_document(&arena, input, &off);
    let mut html = String::new();
    crate::html::format_document(root, &off, &mut html).unwrap();
    let mut xml = String::new();
    crate::xml::format_document(root, &off, &mut xml).unwrap();
    let mut cm = String::new();
    crate::cm::format_document(root, &off, &mut cm).unwrap();
    assert_eq!(html, include_str!("fixtures/link_references.html"));
    assert_eq!(xml, include_str!("fixtures/link_references.xml"));
    assert_eq!(cm, include_str!("fixtures/link_references.cm.md"));
}

/// The tree as option off would give it: `LinkDefinition` dropped, references as the `Link` and
/// `Image` they resolved to. One line per node with sourcepos and literal.
fn resolved_tree(md: &str, leave: bool) -> Vec<String> {
    let arena = Arena::new();
    let root = parse_document(&arena, md, &options(leave));
    let mut out = vec![];
    for edge in root.traverse() {
        let crate::arena_tree::NodeEdge::Start(node) = edge else {
            continue;
        };
        let ast = node.data();
        let depth = node.ancestors().count();
        let line = match &ast.value {
            NodeValue::LinkDefinition(_) => continue,
            NodeValue::Link(l) => format!("link {:?} {:?}", l.url, l.title),
            NodeValue::LinkReference(r) => format!("link {:?} {:?}", r.url, r.title),
            NodeValue::Image(l) => format!("image {:?} {:?}", l.url, l.title),
            NodeValue::ImageReference(r) => format!("image {:?} {:?}", r.url, r.title),
            NodeValue::Text(t) => format!("text {t:?}"),
            v => v.xml_node_name().to_string(),
        };
        out.push(format!("{depth} {line} ({})", ast.sourcepos));
    }
    out
}

/// Byte offset of a 1-based line and byte column.
fn byte_at(md: &str, line: usize, column: usize) -> usize {
    let start: usize = md.split_inclusive('\n').take(line - 1).map(str::len).sum();
    start + column - 1
}

/// Generated documents built from fragments that exercise definitions and references inside
/// paragraphs, block quotes, lists, headings and tables: with the option on, the tree is the
/// option-off tree plus definition nodes and with references kept as such (same positions,
/// same destinations), the HTML is the same, and each definition's sourcepos starts at its `[`
/// and ends on a non-space byte.
#[test]
fn option_on_is_option_off_with_references_kept() {
    const FRAGMENTS: &[&str] = &[
        "[a]: /a\n",
        "[A]: /dup 't'\n",
        "[b c]:\n  </b c>\n  \"T\"\n",
        "[d]: /d\n'bad\n",
        "  [e]: /e   \n",
        "[a]",
        "[a][]",
        "[x][a]",
        "[x][B  C]",
        "![a]",
        "![i][b c]",
        "[nope]",
        "[x][nope]",
        "[*a*]",
        "[x](/inline)",
        "[x][a](/after)",
        "[[a]]",
        "`[a]`",
        "\\[a]",
        "*",
        "text ",
        " ",
        "\n",
        "\n\n",
        "> ",
        "- ",
        "1. ",
        "# ",
        "===\n",
        "| [a] | b |\n|---|---|\n",
        "[^1]",
        "[^1]: note\n",
        "[f]: /f\r\n",
        "\t[g]: /g\n",
        "> [h]: /h\n> ",
        "[é]: /é \"ü\"\n",
        "[É]",
        "    [code]: /c\n",
    ];
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for _ in 0..20_000 {
        let n = 1 + next() % 12;
        let md: String = (0..n)
            .map(|_| FRAGMENTS[(next() % FRAGMENTS.len() as u64) as usize])
            .collect();

        assert_eq!(
            resolved_tree(&md, true),
            resolved_tree(&md, false),
            "input {md:?}"
        );
        let mut on = options(true);
        on.render.sourcepos = true;
        let mut off = options(false);
        off.render.sourcepos = true;
        assert_eq!(
            markdown_to_html(&md, &on),
            markdown_to_html(&md, &off),
            "input {md:?}"
        );

        let arena = Arena::new();
        let root = parse_document(&arena, &md, &options(true));
        for node in root.descendants() {
            let ast = node.data();
            if let NodeValue::LinkDefinition(_) = ast.value {
                let sp = ast.sourcepos;
                let start = byte_at(&md, sp.start.line, sp.start.column);
                let end = byte_at(&md, sp.end.line, sp.end.column);
                assert_eq!(md.as_bytes()[start], b'[', "input {md:?} at {sp}");
                assert!(
                    !md.as_bytes()[end].is_ascii_whitespace(),
                    "input {md:?} at {sp}"
                );
            }
        }
    }
}

/// A grid table cell is parsed by a child parser over fragments of the outer lines: a definition
/// in it lands in the outer tree, mapped to outer positions.
#[cfg(feature = "grid_table")]
#[test]
fn definitions_in_grid_table_cells() {
    let md = "+-----------+\n| [a]: /a   |\n|           |\n| x [a]     |\n+-----------+\n";
    let arena = Arena::new();
    let mut o = options(true);
    o.extension.grid_table = true;
    let root = parse_document(&arena, md, &o);
    let defs: Vec<_> = root
        .descendants()
        .filter(|n| node_matches!(n, NodeValue::LinkDefinition(_)))
        .map(|n| n.data().sourcepos.to_string())
        .collect();
    assert_eq!(defs, ["2:3-2:9"]);
    let refs = root
        .descendants()
        .filter(|n| node_matches!(n, NodeValue::LinkReference(_)))
        .count();
    assert_eq!(refs, 1);
    let mut off = options(false);
    off.extension.grid_table = true;
    assert_eq!(markdown_to_html(md, &o), markdown_to_html(md, &off));
}
