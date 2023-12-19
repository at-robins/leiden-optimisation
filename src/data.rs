//! This module provides types for handling cluster optimisation related data.

use std::collections::{HashMap, HashSet};

use getset::{CopyGetters, Getters};

#[derive(CopyGetters, Getters, Debug)]
/// Cells grouped by cluster with an according resolution.
pub struct ResolutionData {
    /// The resolution used for clustering.
    #[getset(get_copy = "pub")]
    resolution: f64,
    /// The ID of cells grouped by cluster.
    #[getset(get = "pub")]
    clustered_cells: Vec<HashSet<u64>>,
}

impl ResolutionData {
    /// Creates new clustering information with the spcified resolution.
    ///
    /// # Parameters
    ///
    /// * `resolution` - the resolution parameter that has been used during clustering
    /// * `cells` - the cells with according clustering information
    pub fn new<T: AsRef<CellSample>>(resolution: f64, cells: &[T]) -> Self {
        Self {
            resolution,
            clustered_cells: Self::group_by_cluster(cells),
        }
    }

    /// Groups the cells by their respective clusters.
    /// This does not perserve ordering of the clusters.
    ///
    /// # Parameters
    ///
    /// * `cells` - the cells with according clustering information
    pub fn group_by_cluster<T: AsRef<CellSample>>(cells: &[T]) -> Vec<HashSet<u64>> {
        let mut map: HashMap<u64, Vec<u64>> = HashMap::new();
        for cell in cells {
            let cell: &CellSample = cell.as_ref();
            if let Some(grouped_cells) = map.get_mut(&cell.cluster()) {
                grouped_cells.push(cell.id())
            } else {
                map.insert(cell.cluster(), vec![cell.id()]);
            }
        }
        map.into_iter()
            .map(|(_, value)| HashSet::from_iter(value.into_iter()))
            .collect()
    }
}

#[derive(CopyGetters, Debug)]
/// A single cell with according clustering information.
pub struct CellSample {
    /// The ID of the cell.
    #[getset(get_copy = "pub")]
    id: u64,
    /// The cluster the cell belongs to.
    #[getset(get_copy = "pub")]
    cluster: u64,
}

impl CellSample {
    /// Creates a new cell with a specified ID and according clustering information.
    ///
    /// # Parameters
    ///
    /// * `id` - the unique ID (typically a numeric representation of the barcode) of the cell
    /// * `cluster` - the cluster ID the cell belongs to
    pub fn new(id: u64, cluster: u64) -> Self {
        Self { id, cluster }
    }
}

impl AsRef<CellSample> for CellSample {
    fn as_ref(&self) -> &Self {
        &self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_cluster() {
        let clusters = [0u64, 1, 2, 4];
        let cells_per_cluster: u64 = 10;
        let all_cells: Vec<CellSample> = clusters
            .iter()
            .flat_map(|cluster| {
                ((cluster * cells_per_cluster)..((cluster + 1) * cells_per_cluster))
                    .map(|cell_id| CellSample::new(cell_id, *cluster))
            })
            .collect();
        let mut grouped_cells = ResolutionData::group_by_cluster(&all_cells);
        // Cell clusters need to be sorted for the test, so that the order of the vector matches the order of clusters.
        grouped_cells.sort_by(|a, b| a.iter().sum::<u64>().cmp(&b.iter().sum::<u64>()));
        assert_eq!(grouped_cells.len(), clusters.len());
        for (i, cells) in grouped_cells.iter().enumerate() {
            assert_eq!(cells.len(), cells_per_cluster as usize);
            assert!(cells
                .iter()
                .all(|cell| { *cell < (clusters[i] + 1) * cells_per_cluster }))
        }
    }

    #[test]
    fn test_group_by_cluster_empty() {
        let all_cells_empty: Vec<CellSample> = Vec::new();
        let grouped_cells = ResolutionData::group_by_cluster(&all_cells_empty);
        assert!(grouped_cells.is_empty());
    }

    #[test]
    fn test_resolution_data_new() {
        let clusters = [0u64, 1, 2, 4];
        let cells_per_cluster: u64 = 10;
        let all_cells: Vec<CellSample> = clusters
            .iter()
            .flat_map(|cluster| {
                ((cluster * cells_per_cluster)..((cluster + 1) * cells_per_cluster))
                    .map(|cell_id| CellSample::new(cell_id, *cluster))
            })
            .collect();
        let resolution = 0.42;
        let data = ResolutionData::new(resolution, &all_cells);
        assert_eq!(data.resolution(), resolution);
        let mut grouped_cells = data.clustered_cells().clone();
        // Cell clusters need to be sorted for the test, so that the order of the vector matches the order of clusters.
        grouped_cells.sort_by(|a, b| a.iter().sum::<u64>().cmp(&b.iter().sum::<u64>()));
        assert_eq!(grouped_cells.len(), clusters.len());
        for (i, cells) in grouped_cells.iter().enumerate() {
            assert_eq!(cells.len(), cells_per_cluster as usize);
            assert!(cells
                .iter()
                .all(|cell| { *cell < (clusters[i] + 1) * cells_per_cluster }))
        }
    }
}
