//! A simple way of including other files, rust doc and table of content in a markdown file.
//!
//! For a repo's README file, you'll create a _README.tpl.md_ which you can edit like a normal
//! markdown file, but with the added support for fenced includes which are TOML fences with
//! an extra name containing the configuration of the include.
//!
//! ## Example
//!
//! _src/README.tpl.md_:
//! > My title<br>
//! > <br>
//! > Include a table of content:<br>
//! > &#96;&#96;&#96;toml toc<br>
//! > header = "# Table of contents"<br>
//! > &#96;&#96;&#96;<br>
//! > <br>
//! > Extracted from lib.rs' rust doc:<br>
//! > <br>
//! > &#96;&#96;&#96;toml rustdoc<br>
//! > source = "lib.rs"<br>
//! > &#96;&#96;&#96;<br>
//!
//!
//! To generate a _README.md_ file you add a test:
//!
//! ```rust
//! #[test]
//! fn update_readme() {
//!     markdown_includes::update("src/README.tpl.md", "README.md").unwrap();
//! }
//! ```
//!
//! This test will update the README file if necessary, but if running
//! in a CI pipeline (the CI environment variable is set),
//! it will fail if the _README.md_ needs updating.
//!
#[cfg(test)]
mod tests;

mod fence;
mod rustdoc_parse;

use fs_err as fs;
use std::{
    env,
    iter::zip,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use fence::find_fences;

pub fn process_includes_document(document: &mut String, template_dir: &Path) -> Result<()> {
    let mut fences = find_fences(&document, template_dir)?;
    fences.sort_by_key(|f| f.priority());

    for fence in fences {
        fence.run(document)?;
    }
    Ok(())
}

pub fn update<P1: AsRef<Path>, P2: AsRef<Path>>(
    template_file: P1,
    destination_file: P2,
) -> Result<()> {
    let is_ci = env::var("CI").map(|_| true).unwrap_or(false);

    let template_file = template_file.as_ref();

    let template_dir = template_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(""));
    let mut generated_doc = fs::read_to_string(&template_file)
        .context(format!(
            "current working directory: {:?}",
            env::current_dir()
        ))
        .context(format!("failed to read template"))?;
    process_includes_document(&mut generated_doc, &template_dir)?;

    let file = template_file
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    let generated_doc = format!(
        r#"<!-- 
Please don't edit. This document has been generated from {file:?}
--> 
{generated_doc}"#
    );

    let dest_path = destination_file.as_ref();

    let current_doc = if dest_path.exists() {
        fs::read_to_string(&dest_path)?
    } else {
        "".to_string()
    };

    if let Some(diff_str) = diff(&generated_doc, &current_doc) {
        if is_ci {
            bail!(
                "The markdown document {dest_path:?} is out of sync with {template_file:?}. 
            Please re-run the tests and commit the updated file. 
            This message is generated because the test is run on CI (the CI environment variable is set).\n{diff_str}"
            );
        } else {
            fs::write(&dest_path, generated_doc.as_bytes())?;
        }
    }

    Ok(())
}

fn diff(doc1: &str, doc2: &str) -> Option<String> {
    if zip(doc1.lines(), doc2.lines()).any(|(l1, l2)| l1.trim() != l2.trim()) {
        Some(
            zip(doc1.lines(), doc2.lines())
                .filter(|(l1, l2)| l1.trim() != l2.trim())
                .map(|(l1, l2)| format!("> {}\n< {}\n", l1.trim(), l2.trim()))
                .take(5)
                .collect::<Vec<_>>()
                .join(", "),
        )
    } else {
        None
    }
}

#[test]
fn update_readme() {
    update(
        &Path::new("src").join("README.tpl.md"),
        Path::new("README.md"),
    )
    .unwrap();
}
