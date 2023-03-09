mod rustdoc;
mod toc;

use std::path::Path;

use anyhow::Result;
use string_sections::{prelude::Sections, SectionSpan};

use self::{rustdoc::RustDocFence, toc::TocFence};

pub trait Fence {
    /// The fence name is the part after "toml"
    /// <pre>
    /// ```toml name
    /// my fence configuration
    /// ```
    /// </pre>
    /// The name needs to be a minimum of 2 characters long
    ///
    fn is_match(name: &str) -> bool
    where
        Self: Sized;

    /// At which priority the fence should be executed.
    /// The top priority of 10 is reserved for generating
    /// a table of content
    fn priority(self: &Self) -> u8;

    /// create a fence
    ///
    /// - document: the entire document
    /// - section: a fenced section
    fn create(document: &str, section: SectionSpan, template_dir: &Path) -> Result<Box<Self>>
    where
        Self: Sized;

    fn run(self: &Self, document: &mut String) -> Result<()>;
}

fn create_fence(
    document: &str,
    section: SectionSpan,
    template_dir: &Path,
) -> Result<Option<Box<dyn Fence>>> {
    if TocFence::is_match(&section.start_line) {
        Ok(Some(TocFence::create(document, section, template_dir)?))
    } else if RustDocFence::is_match(&section.start_line) {
        Ok(Some(RustDocFence::create(document, section, template_dir)?))
    } else {
        Ok(None)
    }
}

pub fn find_fences(document: &str, template_dir: &Path) -> Result<Vec<Box<dyn Fence>>> {
    let mut fences = Vec::new();

    let section_iter = document.sections(
        |line| line.starts_with("```"),
        |sect| sect.end_line.starts_with("```"),
    );

    for section in section_iter {
        if let Some(fence) = create_fence(document, section, template_dir)? {
            fences.push(fence)
        }
    }
    Ok(fences)
}
