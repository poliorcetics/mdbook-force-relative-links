# `mdbook-force-relative-links`, an `mdbook` pre-processor

[![crates.io](https://img.shields.io/crates/v/mdbook-force-relative-links.svg)](https://crates.io/crates/mdbook-force-relative-links)
[![LICENSE](https://img.shields.io/github/license/poliorcetics/mdbook-force-relative-links.svg)](LICENSE)

This repository contains the source for the `mdbook-force-relative-links` pre-processor,
which will transform all absolute links in a book to a relative form resolving to the same file.

[`mdbook`](https://github.com/rust-lang/mdBook) is a tool to produce books from Markdown files.

[Pre-processors](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html) are programs
that modifies or check a book before is it given to the renderer (to produce HTML, a PDF, an EPUB, ...),
allowing for arbitrary transformations of the resolved Markdown.

## Usage

Add the following to your `book.toml`:

```toml
[prepocessor.force-relative-links]
after = ["links"] # Required to resolve links that comes from `{{#include}}` directives
```

And `mdbook-force-relative-links` needs to be in `PATH` when `mdbook` is called.

## Why ?

It is not rare for a book to be hosted under a subpath of the root,
for example GitHub pages deploys to `<org>.github.io/<project>/<pages>`,
so any absolute link in a book hosted there will resolve to `<org>.github.io`
and not the actual root.

Most of the time, this is not an issue, but it can easily become one when [including](https://rust-lang.github.io/mdBook/format/mdbook.html#including-files) the same file in several places that are not at the same depth (in terms of path):
in that case it is not possible to write the link to correctly resolve in all cases with only `mdbook`.

## License

All the code in this repository is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE] file.
