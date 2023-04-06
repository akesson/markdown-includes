/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use super::Doc;
use anyhow::Context;
use fs_err as fs;
use std::path::Path;

pub fn extract_doc_from_source_file(file_path: impl AsRef<Path>) -> anyhow::Result<Option<Doc>> {
    let source: String = fs::read_to_string(file_path.as_ref())
        .context(format!("cannot open source file {:?}", file_path.as_ref()))?;

    extract_doc_from_source_str(&source)
}

pub fn extract_doc_from_source_str(source: &str) -> anyhow::Result<Option<Doc>> {
    use syn::{parse_str, Lit, Meta, MetaNameValue};

    let ast: syn::File = parse_str(source).context("cannot parse source file")?;
    let mut lines: Vec<String> = Vec::with_capacity(1024);

    for attr in &ast.attrs {
        if Doc::is_toplevel_doc(attr) {
            if let Ok(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lstr),
                ..
            })) = attr.parse_meta()
            {
                let string = &lstr.value();

                match string.lines().count() {
                    0 => lines.push(String::new()),
                    1 => {
                        let line = string.strip_prefix(' ').unwrap_or(string);
                        lines.push(line.to_owned());
                    }

                    // Multiline comment.
                    _ => {
                        fn empty_line(str: &str) -> bool {
                            str.chars().all(char::is_whitespace)
                        }

                        let x = string
                            .lines()
                            .enumerate()
                            .filter(|(i, l)| !(*i == 0 && empty_line(l)))
                            .map(|(_, l)| l);

                        lines.extend(x.map(ToOwned::to_owned));
                    }
                }
            }
        }
    }

    match lines.is_empty() {
        true => Ok(None),
        false => Ok(Some(Doc {
            content: lines.join("\n"),
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_doc_from_source_str_no_doc() {
        let str = indoc! { r#"
            use std::fs;

            struct Nothing {}
            "#
        };

        assert!(extract_doc_from_source_str(str).unwrap().is_none());
    }

    #[test]
    fn test_doc_from_source_str_single_line_comment() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            // normal comment

            //! This is the doc for the crate.
            //!This line doesn't start with space.
            //!
            //! And a nice empty line above us.
            //! Also a line ending in "

            struct Nothing {}
            "#
        };

        let doc = extract_doc_from_source_str(str).unwrap().unwrap();
        let lines: Vec<&str> = doc.content.lines().collect();

        let expected = vec![
            "This is the doc for the crate.",
            "This line doesn't start with space.",
            "",
            "And a nice empty line above us.",
            "Also a line ending in \"",
        ];

        assert_eq!(lines, expected);
    }

    #[test]
    fn test_doc_from_source_str_multi_line_comment() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            /* normal comment */

            /*!
            This is the doc for the crate.
             This line start with space.

            And a nice empty line above us.
            */

            struct Nothing {}
            "#
        };

        let doc = extract_doc_from_source_str(str).unwrap().unwrap();
        let lines: Vec<&str> = doc.content.lines().collect();

        let expected = vec![
            "This is the doc for the crate.",
            " This line start with space.",
            "",
            "And a nice empty line above us.",
        ];

        assert_eq!(lines, expected);
    }

    #[test]
    fn test_doc_from_source_str_single_line_keep_indentation() {
        let str = indoc! { r#"
            #![cfg_attr(not(feature = "std"), no_std)]
            // normal comment

            //! This is the doc for the crate.  This crate does:
            //!
            //!   1. nothing.
            //!   2. niente.

            struct Nothing {}
            "#
        };

        let doc = extract_doc_from_source_str(str).unwrap().unwrap();
        let lines: Vec<&str> = doc.content.lines().collect();

        let expected = vec![
            "This is the doc for the crate.  This crate does:",
            "",
            "  1. nothing.",
            "  2. niente.",
        ];

        assert_eq!(lines, expected);
    }
}
