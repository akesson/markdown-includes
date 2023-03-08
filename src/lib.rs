#[cfg(test)]
mod tests;

mod fence;

use anyhow::Result;
use fence::find_fences;

pub fn process_includes(document: &mut String) -> Result<()> {
    let mut fences = find_fences(&document)?;

    fences.sort_by_key(|f| f.priority());

    for fence in fences {
        fence.run(document)?;
    }
    Ok(())
}
