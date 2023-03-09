use std::ops::Range;

use crate::rustdoc_parse::{parse, RustDocOptions};

use super::Fence;
use anyhow::Result;

pub struct RustDocFence {
    conf: RustDocOptions,
    outer: Range<usize>,
}

impl Fence for RustDocFence {
    fn create(outer: Range<usize>, inner: Range<usize>, document: &str) -> Result<Box<Self>>
    where
        Self: Sized,
    {
        let conf = toml::de::from_str(&document[inner])?;
        println!("opts: {:?}", conf);
        println!("{:?}", std::env::current_dir());
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
