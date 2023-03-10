<!-- 
Please don't edit. This document has been generated from "src/README.tpl.md"
--> 
# markdown-includes

Table of contents

- [markdown-includes](#markdown-includes)
    - [Example](#example)

---

A simple way of including other files, rust doc and table of content in a markdown file.

For a repo's README file, you'll create a _README.tpl.md_ which you can edit like a normal
markdown file, but with the added support for fenced includes which are TOML fences with
an extra name containing the configuration of the include.

## Example

_src/README.tpl.md_:
> My title<br>
> <br>
> Include a table of content:<br>
> &#96;&#96;&#96;toml toc<br>
> header = "# Table of contents"<br>
> &#96;&#96;&#96;<br>
> <br>
> Extracted from lib.rs' rust doc:<br>
> <br>
> &#96;&#96;&#96;toml rustdoc<br>
> source = "lib.rs"<br>
> &#96;&#96;&#96;<br>


To generate a _README.md_ file you add a test:

```rust
#[test]
fn update_readme() {
    markdown_includes::update("src/README.tpl.md", "README.md").unwrap();
}
```

This test will update the README file if necessary, but if running
in a CI pipeline (the CI environment variable is set),
it will fail if the _README.md_ needs updating.

