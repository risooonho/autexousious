use amethyst_utils::removal::Removal;

use MapSelectionEntityId;

/// Marker type for entities to be deleted when the `MapSelectionState` is paused or stopped.
pub type MapSelectionEntity = Removal<MapSelectionEntityId>;