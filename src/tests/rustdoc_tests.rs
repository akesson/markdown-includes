use std::path::Path;

use insta::assert_snapshot;

use crate::process_includes_document;

#[test]
fn test_rustdoc() {
    let doc = r##"
Some markdown

```toml rustdoc
source = "src/tests/rustdoc1.rs"
```

# My h1
text

## My h1.1
## My h1.2

# My h2
"##;

    let mut document = doc.trim().to_string();
    process_includes_document(&mut document, Path::new("")).unwrap();

    assert_snapshot!(document, @r###"
    Some markdown

    Some rust doc here
    ```rust
    let s = String::new();
    ```

    Howdy

    # My h1
    text

    ## My h1.1
    ## My h1.2

    # My h2
    "###);
}
