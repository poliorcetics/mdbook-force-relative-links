use clap::{Arg, ArgMatches, Command};
use mdbook_preprocessor::Preprocessor;
use mdbook_preprocessor::errors::Result;
use semver::{Version, VersionReq};

mod force_relative_links;

pub fn make_app() -> Command {
    Command::new(force_relative_links::NAME)
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
    let preprocessor = force_relative_links::ForceRelativeLinks;

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(std::io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook_preprocessor::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
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
    let supported = pre.supports_renderer(renderer).unwrap_or_else(|err| {
        eprintln!("Failed to detect renderer support: {err:?}");
        false
    });

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
