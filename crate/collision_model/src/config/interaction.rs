use derive_new::new;
use serde::{Deserialize, Serialize};
use shape_model::Volume;

use crate::config::InteractionKind;

/// Effects of one object on another
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq, Serialize, new)]
#[serde(deny_unknown_fields)]
pub struct Interaction {
    /// Type of collision -- impact, picking weapon, grabbing, and so on.
    #[serde(flatten)]
    pub kind: InteractionKind,
    /// Effect volume.
    pub bounds: Vec<Volume>,
    /// Whether this will hit multiple objects. Defaults to `false`.
    #[serde(default)]
    pub multiple: bool,
}
