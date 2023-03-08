use insta::assert_snapshot;

use crate::process_includes;

#[test]
fn test_toc() {
    let doc = r##"
Some markdown

```toml toc
header = "# Table of contents"
```

# My h1
text

## My h1.1
## My h1.2

# My h2
"##;

    let mut document = doc.trim().to_string();
    process_includes(&mut document).unwrap();

    assert_snapshot!(document, @r###"
    Some markdown

    # Table of contents

    - [My h1](#my-h1)
        - [My h1.1](#my-h1.1)
        - [My h1.2](#my-h1.2)
    - [My h2](#my-h2)

    # My h1
    text

    ## My h1.1
    ## My h1.2

    # My h2
    "###);
}
