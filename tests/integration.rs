use std::path::Path;
use std::process::Command;

#[test]
fn called_through_mdbook() {
    // Set by Cargo for integration tests.
    let bin_path = env!("CARGO_BIN_EXE_mdbook-force-relative-links");
    let bin_dir = Path::new(bin_path).parent().unwrap();

    let book_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-books/book-1");
    let out_dir = tempfile::tempdir().unwrap();

    // So mdbook can find the preprocessor
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let status = Command::new("mdbook")
        .args(["build", "--dest-dir"])
        .arg(out_dir.path())
        .arg(&book_dir)
        .env("PATH", &path)
        .status()
        .expect("mdbook must be available in PATH");

    assert!(status.success(), "mdbook build failed");

    // chapter_1.html is at the root level — absolute /chapter_2/... should
    // have been rewritten to the relative chapter_2/...
    let chapter_1 = std::fs::read_to_string(out_dir.path().join("chapter_1.html")).unwrap();
    assert!(
        chapter_1.contains("chapter_2/chapter_2.1.html"),
        "expected relative link in chapter_1.html"
    );
    assert!(
        !chapter_1.contains("\"/chapter_2"),
        "unexpected absolute link in chapter_1.html"
    );

    // chapter_2/chapter_2.1.html is one level deep — absolute /chapter_1.md
    // should have been rewritten to ../chapter_1.html
    let chapter_2_1 =
        std::fs::read_to_string(out_dir.path().join("chapter_2/chapter_2.1.html")).unwrap();
    assert!(
        chapter_2_1.contains("../chapter_1.html"),
        "expected relative link in chapter_2.1.html"
    );
    assert!(
        !chapter_2_1.contains("\"/chapter_1"),
        "unexpected absolute link in chapter_2.1.html"
    );
}
