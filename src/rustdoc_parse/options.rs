/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

use serde::Deserialize;

use super::transform::IntralinksConfig;

#[derive(PartialEq, Eq, Debug, Deserialize)]
pub struct RustDocOptions {
    pub source: PathBuf,
    #[serde(default)]
    pub workspace_project: Option<String>,
    #[serde(default)]
    pub intralinks: Option<IntralinksConfig>,
}
