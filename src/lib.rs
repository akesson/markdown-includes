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

use std::{env, fs};

use anyhow::{bail, Result};
use camino::Utf8PathBuf;
use fence::find_fences;

pub fn process_includes_document(document: &mut String) -> Result<()> {
    let mut fences = find_fences(&document)?;
    fences.sort_by_key(|f| f.priority());

    for fence in fences {
        fence.run(document)?;
    }
    Ok(())
}

pub fn update(template_file: &str, destination_file: &str) -> Result<()> {
    let is_ci = env::var("CI").map(|_| true).unwrap_or(false);

    let template_path = Utf8PathBuf::from(template_file);
    let mut generated_doc = fs::read_to_string(&template_path)?;
    process_includes_document(&mut generated_doc)?;
    let generated_doc = format!(
        r#"<!-- 
Please don't edit. This document has been generated from {template_file}
--> 
{generated_doc}"#
    );

    let dest_path = Utf8PathBuf::from(destination_file);

    let current_doc = if dest_path.exists() {
        fs::read_to_string(&dest_path)?
    } else {
        "".to_string()
    };

    if generated_doc != current_doc {
        if is_ci {
            let diff = diff::lines(&current_doc, &generated_doc)
                .iter()
                .map(|diff| match diff {
                    diff::Result::Left(l) => format!("-{}", l),
                    diff::Result::Both(l, _) => format!(" {}", l),
                    diff::Result::Right(r) => format!("+{}", r),
                })
                .collect::<Vec<_>>()
                .join("\n");

            bail!(
                "The markdown document {dest_path} is out of sync with {template_path}. 
            Please re-run the tests and commit the updated file. 
            This message is generated because the test is run on CI (the CI environment variable is set).\n{diff}"
            );
        } else {
            fs::write(&dest_path, generated_doc.as_bytes())?;
        }
    }

    Ok(())
}

#[test]
fn update_readme() {
    update("src/README.tpl.md", "README.md").unwrap();
}
