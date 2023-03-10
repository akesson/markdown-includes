/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use anyhow::{anyhow, Context};
use fs_err as fs;
use std::cell::Cell;
use std::path::Path;

mod extract_doc;
mod options;
pub mod transform;
pub mod utils;

pub use extract_doc::extract_doc_from_source_file;

pub use self::options::RustDocOptions;

pub fn parse(options: &options::RustDocOptions) -> anyhow::Result<String> {
    let project: Project = match options.workspace_project {
        None => Project::from_current_dir()?,
        Some(ref project) => Project::from_current_dir_workspace_project(project)?,
    };
    let entryfile: &Path = &options.source;

    let doc: Doc = extract_doc_from_source_file(entryfile)?
        .ok_or_else(|| anyhow!("crate-level rustdoc not found"))?;

    let doc = transform_doc(&doc, &project, entryfile, &options)?;
    Ok(doc.content)
}

fn transform_doc(
    doc: &Doc,
    project: &Project,
    entrypoint: impl AsRef<Path>,
    options: &options::RustDocOptions,
) -> anyhow::Result<Doc> {
    use transform::{
        DocTransform, DocTransformIntralinks, DocTransformRustMarkdownTag,
        DocTransformRustRemoveComments,
    };

    let transform = DocTransformRustRemoveComments::new();
    // TODO Use `into_ok()` once it is stable (https://github.com/rust-lang/rust/issues/61695).
    let doc = transform.transform(doc)?;

    let transform = DocTransformRustMarkdownTag::new();
    // TODO Use `into_ok()` once it is stable (https://github.com/rust-lang/rust/issues/61695).
    let doc = transform.transform(&doc)?;

    let had_warnings = Cell::new(false);
    let transform = DocTransformIntralinks::new(
        project.get_package_name(),
        entrypoint,
        |msg| {
            println!("{}", msg);
            had_warnings.set(true);
        },
        options.intralinks.clone(),
    );

    Ok(transform.transform(&doc)?)
}

#[derive(PartialEq, Eq, Debug)]
struct Project {
    package_name: String,
}

impl Project {
    /// Creates a [`Project`] the current directory.  It will search ancestor paths until it finds
    /// the root of the project.
    pub fn from_current_dir() -> anyhow::Result<Project> {
        let cmd = cargo_metadata::MetadataCommand::new();
        let metadata = cmd.exec()?;
        let package = metadata
            .root_package()
            .context("project has no root package")?;

        Ok(Project::from_package(package))
    }

    fn select_package<'a>(
        metadata: &'a cargo_metadata::Metadata,
        package_name: &str,
    ) -> Option<&'a cargo_metadata::Package> {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.name == package_name)?;

        // We need to make sure the package we found is actually a project of the workspace.
        match metadata.workspace_members.contains(&package.id) {
            false => None,
            true => Some(package),
        }
    }

    pub fn from_current_dir_workspace_project(project_name: &str) -> anyhow::Result<Project> {
        let cmd = cargo_metadata::MetadataCommand::new();
        let metadata = cmd.exec()?;

        let package =
            Project::select_package(&metadata, project_name).context("project has no package")?;

        Ok(Project::from_package(package))
    }

    fn from_package(package: &cargo_metadata::Package) -> Project {
        const LIB_CRATE_KINDS: [&str; 6] =
            ["lib", "dylib", "staticlib", "cdylib", "rlib", "proc-macro"];
        let lib_packages: Vec<&cargo_metadata::Target> = package
            .targets
            .iter()
            .filter(|target| {
                target
                    .kind
                    .iter()
                    .any(|k| LIB_CRATE_KINDS.contains(&k.as_str()))
            })
            .collect();

        assert!(lib_packages.len() <= 1, "more than one lib target");

        Project {
            package_name: package.name.clone(),
        }
    }

    #[must_use]
    pub fn get_package_name(&self) -> &str {
        &self.package_name
    }
}

fn project_package_name(manifest_path: impl AsRef<Path>) -> Option<String> {
    let str: String = fs::read_to_string(&manifest_path).ok()?;
    let toml: toml::Value = toml::from_str(&str).ok()?;
    let package_name = toml
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(toml::Value::as_str)?;

    Some(package_name.to_owned())
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Doc {
    pub content: String,
}

impl Doc {
    // TODO implement FromStr when ! type is stable.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl Into<String>) -> Doc {
        Doc {
            content: str.into(),
        }
    }

    fn is_toplevel_doc(attr: &syn::Attribute) -> bool {
        use syn::token::Bang;
        use syn::AttrStyle;

        attr.style == AttrStyle::Inner(Bang::default()) && attr.path.is_ident("doc")
    }

    // Return the markdown as a string.  Note that the line terminator will always be a line feed.
    #[must_use]
    pub fn as_string(&self) -> &str {
        &self.content
    }
}
