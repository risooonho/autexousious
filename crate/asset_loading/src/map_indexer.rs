use std::path::Path;

use game_model::config::AssetRecord;

use {AssetIndexingUtils, DirTraverse};

/// Indexes map assets.
#[derive(Debug)]
pub struct MapIndexer;

impl MapIndexer {
    /// Returns `AssetRecords` for each of the maps in the namespace.
    ///
    /// # Parameters
    ///
    /// * `namespace`: Namespace that the maps reside in.
    /// * `maps_dir`: Directory containing maps' assets.
    pub fn index(namespace: &str, maps_dir: &Path) -> Vec<AssetRecord> {
        DirTraverse::child_directories(&maps_dir)
            .into_iter()
            .filter_map(|object_dir| {
                AssetIndexingUtils::asset_record(namespace.to_string(), object_dir)
            }).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::PathBuf;

    use game_model::config::{AssetRecord, AssetSlugBuilder};
    use hamcrest::prelude::*;
    use tempfile::tempdir;

    use super::MapIndexer;

    #[test]
    fn returns_asset_record_for_each_map() -> io::Result<()> {
        let maps_tempdir = tempdir()?;
        let maps_dir = maps_tempdir.path();

        let map_0_dir = maps_dir.join("map_0");
        let map_1_dir = maps_dir.join("map_1");
        [&map_0_dir, &map_1_dir]
            .iter()
            .fold(Ok(()), |result, dir| {
                result.and_then(|_| fs::create_dir(&dir))
            })?;

        assert_that!(
            &MapIndexer::index("rara", &maps_dir),
            contains(vec![
                asset_record("rara", "map_0", map_0_dir),
                asset_record("rara", "map_1", map_1_dir),
            ]).exactly()
        );

        Ok(())
    }

    fn asset_record(namespace: &str, name: &str, path: PathBuf) -> AssetRecord {
        AssetRecord {
            asset_slug: AssetSlugBuilder::default()
                .namespace(namespace.to_string())
                .name(name.to_string())
                .build()
                .expect("Failed to build asset slug."),
            path,
        }
    }
}