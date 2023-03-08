mod rustdoc;
mod toc;

use std::ops::Range;

use anyhow::Result;
use line_span::LineSpans;

use self::toc::TocFence;

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
    /// - outer: the range of the fence declaration (including the fence ticks)
    /// - inner: the range of the fence configuration (the inside part)
    /// - document: the entire document
    fn create(outer: Range<usize>, inner: Range<usize>, document: &str) -> Result<Box<Self>>
    where
        Self: Sized;

    fn run(self: &Self, document: &mut String) -> Result<()>;
}

fn create_fence(
    outer: Range<usize>,
    inner: Range<usize>,
    document: &str,
    name: &str,
) -> Result<Option<Box<dyn Fence>>> {
    if TocFence::is_match(name) {
        Ok(Some(TocFence::create(outer, inner, document)?))
    } else {
        Ok(None)
    }
}

pub fn find_fences(document: &str) -> Result<Vec<Box<dyn Fence>>> {
    let mut header: Option<Range<usize>> = None;
    let mut fences = Vec::new();
    for line in document.line_spans() {
        if line.starts_with("```toml ") && line.len() >= 10 && header.is_none() {
            header = Some(line.range());
        } else if line.starts_with("```") {
            if let Some(header) = &header {
                let name = &document[header.start + 8..header.end];
                let outer = header.start..line.end();
                let inner = header.end..line.start();
                if let Some(fence) = create_fence(outer, inner, document, name)? {
                    fences.push(fence);
                }
            }
        }
    }
    Ok(fences)
}
