use mdbook_markdown::MarkdownOptions;
use mdbook_markdown::pulldown_cmark::{Event, Tag};
use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

/// Name of this preprocessor.
pub const NAME: &str = "mdbook-force-relative-links";

/// A preprocessor which converts absolute links to relative ones.
///
/// Works with all renderers
pub struct ForceRelativeLinks;

impl Preprocessor for ForceRelativeLinks {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(handle_item);
        Ok(book)
    }
}

fn handle_item(item: &mut BookItem) {
    let BookItem::Chapter(chapter) = item else {
        return;
    };

    let Some(path) = chapter.path.as_deref() else {
        return;
    };

    // Don't count the file itself and its immediate parent dir, else we'll go out of the source directory.
    let parent_count = path.ancestors().count().saturating_sub(2);
    let prefix = "../".repeat(parent_count);

    let events = mdbook_markdown::new_cmark_parser(&chapter.content, &MarkdownOptions::default())
        .map(|event| handle_link(event, &prefix));

    // Replace the chapter content with the fixed links.
    let mut buf = String::with_capacity(chapter.content.len());
    pulldown_cmark_to_cmark::cmark(events, &mut buf)
        .expect("Markdown serialization has breen broken by the preprocessor");
    chapter.content = buf;
}

fn handle_link<'a>(mut event: Event<'a>, prefix: &str) -> Event<'a> {
    if let Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) = &mut event {
        // Ignore any non-absolute (and non-local) link
        if dest_url.starts_with('/') {
            *dest_url = format!("{prefix}{}", dest_url.trim_start_matches('/')).into();
        }
    };

    event
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn preprocessor_run() {
        let input_json = r##"[
            {
                "root": "/path/to/book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "description": null,
                        "language": "en",
                        "text-direction": null,
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "nop": {}
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.21"
            },
            {
                "items": [
                    {
                        "Chapter": {
                            "name": "Chapter 1",
                            "content": "# Chapter 1\n\n[link 0001](/chapter_2/chapter_2.1.md)\n",
                            "number": [
                                1
                            ],
                            "sub_items": [],
                            "path": "chapter_1.md",
                            "source_path": "chapter_1.md",
                            "parent_names": []
                        }
                    },
                    {
                        "Chapter": {
                            "name": "Chapter 2.1",
                            "content": "# Chapter 2.1\n\n[link 0000](/chapter_1.md)\n\n[link 0001](/chapter_2/chapter_2.1.md)\n\n![image 0001](/images/image.png)\n",
                            "number": [
                                2,
                                1
                            ],
                            "sub_items": [],
                            "path": "chapter_2/chapter_2.1.md",
                            "source_path": "chapter_2/chapter_2.1.md",
                            "parent_names": [
                                "Chapter 2"
                            ]
                        }
                    },
                    {
                        "Chapter": {
                           "name": "Chapter 2",
                           "content": "# Chapter 2\n",
                           "number": [
                               2
                           ],
                           "sub_items": [
                                {
                                    "Chapter": {
                                        "name": "Chapter 2.1",
                                        "content": "# Chapter 2.1\n\n[link 0000](../chapter_1.md)\n\n[link 0001](../chapter_2/chapter_2.1.md)",
                                        "number": [
                                            2,
                                            1
                                        ],
                                        "sub_items": [],
                                        "path": "chapter_2/chapter_2.1.md",
                                        "source_path": "chapter_2/chapter_2.1.md",
                                        "parent_names": [
                                            "Chapter 2"
                                        ]
                                    }
                                }
                           ],
                           "path": "chapter_2.md",
                           "source_path": "chapter_2.md",
                           "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]"##;
        let input_json = input_json.as_bytes();

        let expected_json = r##"[
            {
                "root": "/path/to/book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "language": "en",
                        "src": "src",
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "nop": {}
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.21"
            },
            {
                "items": [
                    {
                        "Chapter": {
                            "name": "Chapter 1",
                            "content": "# Chapter 1\n\n[link 0001](chapter_2/chapter_2.1.md)",
                            "number": [
                                1
                            ],
                            "sub_items": [],
                            "path": "chapter_1.md",
                            "source_path": "chapter_1.md",
                            "parent_names": []
                        }
                    },
                    {
                        "Chapter": {
                            "name": "Chapter 2.1",
                            "content": "# Chapter 2.1\n\n[link 0000](../chapter_1.md)\n\n[link 0001](../chapter_2/chapter_2.1.md)\n\n![image 0001](../images/image.png)",
                            "number": [
                                2,
                                1
                            ],
                            "sub_items": [],
                            "path": "chapter_2/chapter_2.1.md",
                            "source_path": "chapter_2/chapter_2.1.md",
                            "parent_names": [
                                "Chapter 2"
                            ]
                        }
                    },
                    {
                        "Chapter": {
                           "name": "Chapter 2",
                           "content": "# Chapter 2",
                           "number": [
                               2
                           ],
                           "sub_items": [
                                {
                                    "Chapter": {
                                        "name": "Chapter 2.1",
                                        "content": "# Chapter 2.1\n\n[link 0000](../chapter_1.md)\n\n[link 0001](../chapter_2/chapter_2.1.md)",
                                        "number": [
                                            2,
                                            1
                                        ],
                                        "sub_items": [],
                                        "path": "chapter_2/chapter_2.1.md",
                                        "source_path": "chapter_2/chapter_2.1.md",
                                        "parent_names": [
                                            "Chapter 2"
                                        ]
                                    }
                                }
                           ],
                           "path": "chapter_2.md",
                           "source_path": "chapter_2.md",
                           "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]"##;
        let expected_json = expected_json.as_bytes();

        let (ctx, book) = mdbook_preprocessor::parse_input(input_json).unwrap();

        let result = ForceRelativeLinks.run(&ctx, book);
        assert!(result.is_ok());

        let (_, expected_book) = mdbook_preprocessor::parse_input(expected_json).unwrap();

        // The preprocessor should have changed the links in to the book content.
        let actual_book = result.unwrap();
        pretty_assertions::assert_eq!(actual_book, expected_book);
    }
}
