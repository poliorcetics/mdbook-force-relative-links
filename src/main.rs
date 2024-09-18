use clap::{Arg, ArgMatches, Command};
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use pulldown_cmark::{Event, Tag};
use semver::{Version, VersionReq};

/// Name of this preprocessor.
const NAME: &str = "mdbook-force-relative-links";

pub fn make_app() -> Command {
    Command::new(NAME)
        .about("An mdbook preprocessor which converts absolute links to relative ones")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}
fn main() {
    let matches = make_app().get_matches();

    // Users will want to construct their own preprocessor here
    let preprocessor = ForceRelativeLinks;

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(std::io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(std::io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

/// A preprocessor which converts absolute links to relative ones.
///
/// Works with all renderers
pub struct ForceRelativeLinks;

impl Preprocessor for ForceRelativeLinks {
    fn name(&self) -> &str {
        NAME
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
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

    // eprintln!("preprocessor: {NAME}: handling '{}'", path.display());

    // Don't count the file itself and its immediate parent dir, else we'll go out of the source directory.
    let parent_count = path.ancestors().count().saturating_sub(2);
    let prefix = "../".repeat(parent_count);

    let events = mdbook::utils::new_cmark_parser(&chapter.content, false)
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
            // eprintln!("preprocessor: {NAME}: fixed '{dest_url}' to '{fixed_dest_url}'");
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
                        "language": "en",
                        "multilingual": false,
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
                "sections": [
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
                        "multilingual": false,
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
                "sections": [
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

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();

        let result = ForceRelativeLinks.run(&ctx, book);
        assert!(result.is_ok());

        let (_, expected_book) =
            mdbook::preprocess::CmdPreprocessor::parse_input(expected_json).unwrap();

        // The preprocessor should have changed the links in to the book content.
        let actual_book = result.unwrap();
        pretty_assertions::assert_eq!(actual_book, expected_book);
    }
}
