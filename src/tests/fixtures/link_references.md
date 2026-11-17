# Link references

A full [reference][Foo Bar], a collapsed [foo bar][] and a shortcut [Foo bar].
An image ![full][img], ![img][] and ![img]; an [unresolved][nope] and [nope].
Nested [*emphasis* `code`][foo bar] and ![alt *x*][IMG].
A [link [inner] text][foo bar] and [shortcut [foo bar]].
Inline [x](/inline "t") stays a link; [x][foo bar](/after) too.
A footnote[^1] and a [^nope] shortcut.

[foo bar]: /url "Title"
[img]: </img.png> 'Image title'
  [indented]:
  /next-line
  (paren title)
[foo bar]: /duplicate
[^1]: The note.

Text right after.

> [quoted]: /q
> A [quoted] reference.

- [item]: /i
- [item] in a list.

[setext]: /s
Heading [setext]
===

| a | b |
|---|---|
| [foo bar] | [img] |

[escaped \] label]: /e
[escaped \] label] and [ÄÖ]; [äö] and [ẞ].

[ÄÖ]: /umlaut
[ss]: /eszett
