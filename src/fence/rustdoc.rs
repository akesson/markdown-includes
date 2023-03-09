use std::{ops::Range, path::Path};

use crate::rustdoc_parse::{parse, RustDocOptions};

use super::Fence;
use anyhow::Result;
use string_sections::SectionSpan;

pub struct RustDocFence {
    conf: RustDocOptions,
    outer: Range<usize>,
}

impl Fence for RustDocFence {
    fn create(document: &str, section: SectionSpan, template_dir: &Path) -> Result<Box<Self>>
    where
        Self: Sized,
    {
        let mut conf: RustDocOptions = toml::de::from_str(&document[section.inner_range()])?;
        conf.source = template_dir.join(&conf.source);
        let outer = section.outer_range();
        Ok(Box::new(Self { conf, outer }))
    }

    fn is_match(name: &str) -> bool
    where
        Self: Sized,
    {
        name.to_lowercase().ends_with("rustdoc")
    }

    fn priority(self: &Self) -> u8 {
        1
    }

    fn run(self: &Self, document: &mut String) -> Result<()> {
        let content = match parse(&self.conf) {
            Ok(rustdoc) => rustdoc,
            Err(e) => format!("```toml rustdoc\n{e}\n```"),
        };

        document.replace_range(self.outer.clone(), &content);
        Ok(())
    }
}
