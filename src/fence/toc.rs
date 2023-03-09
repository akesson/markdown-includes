use anyhow::Result;
use percent_encoding::{percent_encode, CONTROLS};
use serde::Deserialize;
use std::{ops::Range, str::FromStr};
use string_sections::SectionSpan;

use super::Fence;

pub struct TocFence {
    conf: TocConfig,
    outer: Range<usize>,
}

impl Fence for TocFence {
    fn is_match(name: &str) -> bool
    where
        Self: Sized,
    {
        let name = name.to_lowercase();
        name.ends_with("toc") || name.ends_with("table-of-contents")
    }

    fn priority(self: &Self) -> u8 {
        10
    }

    fn create(document: &str, section: SectionSpan) -> Result<Box<Self>>
    where
        Self: Sized,
    {
        let conf = toml::de::from_str(&document[section.inner_range()])?;
        let outer = section.outer_range();
        Ok(Box::new(Self { conf, outer }))
    }

    fn run(self: &Self, document: &mut String) -> Result<()> {
        let mut output = String::new();

        if let Some(ref header) = self.conf.header {
            output.push_str(&header);
            output.push_str("\n\n");
        }

        let headings = find_headings(document);
        let toc = headings
            .iter()
            .filter_map(|h| h.format(&self.conf))
            .collect::<Vec<String>>()
            .join("\n");

        output.push_str(&toc);
        document.replace_range(self.outer.clone(), &output);
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct TocConfig {
    #[serde(default = "default_bullet")]
    pub bullet: String,
    #[serde(default = "default_inline")]
    pub indent: usize,
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub min_depth: usize,
    pub header: Option<String>,
    #[serde(default = "default_link")]
    pub link: bool,
}

fn default_link() -> bool {
    true
}
fn default_inline() -> usize {
    4
}
fn default_bullet() -> String {
    "-".to_string()
}
fn slugify(text: &str) -> String {
    percent_encode(text.replace(" ", "-").to_lowercase().as_bytes(), CONTROLS).to_string()
}

pub fn find_headings(content: &str) -> Vec<Heading> {
    let mut in_fence = false;

    content
        .lines()
        .filter(|line| {
            let was_inside = in_fence;
            line.starts_with("```").then(|| in_fence = !in_fence);
            !(was_inside || in_fence)
        })
        .map(Heading::from_str)
        .filter_map(Result::ok)
        .collect::<Vec<Heading>>()
}

pub struct Heading {
    pub depth: usize,
    pub title: String,
}

impl FromStr for Heading {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim_end();
        if trimmed.starts_with("#") {
            let mut depth = 0usize;
            let title = trimmed
                .chars()
                .skip_while(|c| {
                    if *c == '#' {
                        depth += 1;
                        true
                    } else {
                        false
                    }
                })
                .collect::<String>()
                .trim_start()
                .to_owned();
            Ok(Heading {
                depth: depth - 1,
                title,
            })
        } else {
            Err(())
        }
    }
}

impl Heading {
    pub fn format(&self, config: &TocConfig) -> Option<String> {
        if self.depth >= config.min_depth
            && config.max_depth.map(|d| self.depth <= d).unwrap_or(true)
        {
            let head = format!(
                "{}{} {}",
                " ".repeat(config.indent)
                    .repeat(self.depth - config.min_depth),
                &config.bullet,
                if !config.link {
                    self.title.clone()
                } else {
                    format!("[{}](#{})", &self.title, slugify(&self.title))
                }
            );
            println!("t: {}", head);
            Some(head)
        } else {
            None
        }
    }
}
