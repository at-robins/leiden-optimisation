//! This module provides types for handling cluster optimisation related data.

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
};

use getset::{CopyGetters, Getters};

use crate::optimisation::{cluster_overlaps_relative, cluster_stability};

#[derive(CopyGetters, Getters, Clone, Debug)]
/// Cells grouped by cluster with an according resolution.
pub struct Cluster {
    /// The original identifier of the cluster (as specified in the input file).
    #[getset(get_copy = "pub")]
    cluster_id: usize,
    /// The absolute number of cells in all clusters.
    #[getset(get_copy = "pub")]
    total_cell_count: usize,
    /// The ID of cells in this cluster.
    #[getset(get = "pub")]
    cells: HashSet<usize>,
}

impl Cluster {
    /// Creates new cell cluster.
    ///
    /// # Parameters
    ///
    /// * `cluster_id` - the original identifier of the cluster (as specified in the input file)
    /// * `cells` - the cells belonging to the cluster
    /// * `total_cell_count` - the number of cells in clusters combined
    pub fn new(cluster_id: usize, cells: HashSet<usize>, total_cell_count: usize) -> Self {
        Self {
            cluster_id,
            total_cell_count,
            cells,
        }
    }

    /// Returns the absolute size (number of cells) of this cluster.
    pub fn absolute_cluster_size(&self) -> usize {
        self.cells().len()
    }

    /// Returns the relative cluster size compared to all other clusters of the same [`ResolutionData`].
    pub fn relative_cluster_size(&self) -> f64 {
        (self.cells().len() as f64) / (self.total_cell_count() as f64)
    }

    /// Returns the best matching parent population based on the specified populations
    /// or an error if no parent populations have been specified.
    ///
    /// # Parameters
    ///
    /// * `potential_parents` - the potential parent clusters
    pub fn best_parent<T: Borrow<Cluster>>(
        &self,
        potential_parents: &[T],
    ) -> Result<usize, &'static str> {
        let potential_parent_cell_clusters: Vec<&HashSet<usize>> = potential_parents
            .iter()
            .map(|cluster| cluster.borrow().cells())
            .collect();

        potential_parents
            .iter()
            .map(|cluster| cluster.borrow().cluster_id())
            .zip(cluster_overlaps_relative(&potential_parent_cell_clusters, self.cells())?)
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .expect("The relative cluster overlap must be a valid number.")
            })
            .map(|value| value.0)
            .ok_or("No parent clusters have been supplied.")
    }
}

#[derive(CopyGetters, Getters, Debug)]
/// Cells grouped by cluster with an according resolution.
pub struct ResolutionData {
    /// The resolution used for clustering.
    #[getset(get_copy = "pub")]
    resolution: f64,
    /// The ID of cells grouped by cluster.
    #[getset(get = "pub")]
    clustered_cells: Vec<Cluster>,
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

    /// Returns the number of clusters the cells are grouped into.
    pub fn clusters(&self) -> usize {
        self.clustered_cells.len()
    }

    /// Groups the cells by their respective clusters.
    /// This does not perserve ordering of the clusters.
    ///
    /// # Parameters
    ///
    /// * `cells` - the cells with according clustering information
    pub fn group_by_cluster<T: AsRef<CellSample>>(cells: &[T]) -> Vec<Cluster> {
        let total_cell_number = cells.len();
        let mut map: HashMap<usize, Vec<usize>> = HashMap::new();
        for cell in cells {
            let cell: &CellSample = cell.as_ref();
            if let Some(grouped_cells) = map.get_mut(&cell.cluster()) {
                grouped_cells.push(cell.id())
            } else {
                map.insert(cell.cluster(), vec![cell.id()]);
            }
        }
        map.into_iter()
            .map(|(cluster_id, value)| {
                Cluster::new(cluster_id, HashSet::from_iter(value.into_iter()), total_cell_number)
            })
            .collect()
    }
}

#[derive(CopyGetters, Debug)]
/// A single cell with according clustering information.
pub struct CellSample {
    /// The ID of the cell.
    #[getset(get_copy = "pub")]
    id: usize,
    /// The cluster the cell belongs to.
    #[getset(get_copy = "pub")]
    cluster: usize,
}

impl CellSample {
    /// Creates a new cell with a specified ID and according clustering information.
    ///
    /// # Parameters
    ///
    /// * `id` - the unique ID (typically a numeric representation of the barcode) of the cell
    /// * `cluster` - the cluster ID the cell belongs to
    pub fn new(id: usize, cluster: usize) -> Self {
        Self { id, cluster }
    }
}

impl AsRef<CellSample> for CellSample {
    fn as_ref(&self) -> &Self {
        &self
    }
}

#[derive(CopyGetters, Getters, Debug)]
/// Data associated with the stability of clusters observed at a specific resolution.
pub struct ClusterStabilityData {
    /// The number of clusters in the parent clustering.
    #[getset(get_copy = "pub")]
    clusters_parent: usize,
    /// The number of clusters in the child clustering.
    #[getset(get_copy = "pub")]
    clusters_child: usize,
    /// The resolution the parent clustering was performed at.
    #[getset(get_copy = "pub")]
    parent_resolution: f64,
    /// The resolution the child clustering was performed at.
    #[getset(get_copy = "pub")]
    child_resolution: f64,
    /// The stabilities of each child cluster.
    #[getset(get = "pub")]
    stabilities: Vec<f64>,
}

impl ClusterStabilityData {
    /// Creates clustering stability data from  two different clusterings performed at different resolutions.
    /// Returns an error if the number of clusters present in both datasets is identical,
    /// as the number of clusters present is used to determine the parent-child-relation of the data.
    ///
    /// # Parameters
    ///
    /// * `clustering_a` - the first clustering data with a specific resolution
    /// * `clustering_b` - the second clustering data with a specific resolution
    pub fn from_clustering(
        clustering_a: &ResolutionData,
        clustering_b: &ResolutionData,
    ) -> Result<Self, String> {
        // Determines the parent child relation of the data based on the number of clusters.
        let (parent_data, child_data) = if clustering_a.clusters() < clustering_b.clusters() {
            (clustering_a, clustering_b)
        } else if clustering_a.clusters() > clustering_b.clusters() {
            (clustering_b, clustering_a)
        } else {
            return Err("The number of clusters is identical.".to_string());
        };
        // Calcultes the stabilites.
        let stabilities = child_data
            .clustered_cells()
            .iter()
            .map(|cluster_child| {
                cluster_stability(
                    &parent_data
                        .clustered_cells()
                        .iter()
                        .map(Cluster::cells)
                        .collect::<Vec<&HashSet<usize>>>(),
                    cluster_child.cells(),
                )
                .expect("The child cluster cannot be empty at this point.")
            })
            .collect();
        Ok(Self {
            clusters_parent: parent_data.clusters(),
            clusters_child: child_data.clusters(),
            parent_resolution: parent_data.resolution(),
            child_resolution: child_data.resolution(),
            stabilities,
        })
    }

    /// Returns the mean stability of all child clusters.
    pub fn mean_stability(&self) -> f64 {
        self.stabilities().iter().sum::<f64>() / (self.stabilities().len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn test_group_by_cluster() {
        let clusters = [0usize, 1, 2, 4];
        let cells_per_cluster: usize = 10;
        let all_cells: Vec<CellSample> = clusters
            .iter()
            .flat_map(|cluster| {
                ((cluster * cells_per_cluster)..((cluster + 1) * cells_per_cluster))
                    .map(|cell_id| CellSample::new(cell_id, *cluster))
            })
            .collect();
        let mut grouped_cells = ResolutionData::group_by_cluster(&all_cells);
        // Cell clusters need to be sorted for the test, so that the order of the vector matches the order of clusters.
        grouped_cells.sort_by(|a, b| {
            a.cells()
                .iter()
                .sum::<usize>()
                .cmp(&b.cells().iter().sum::<usize>())
        });
        assert_eq!(grouped_cells.len(), clusters.len());
        for (i, cell_cluster) in grouped_cells.iter().enumerate() {
            let cells = cell_cluster.cells();
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
        let clusters = [0usize, 1, 2, 4];
        let cells_per_cluster: usize = 10;
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
        assert_eq!(data.clusters(), clusters.len());
        let mut grouped_cells = data.clustered_cells().clone();
        // Cell clusters need to be sorted for the test, so that the order of the vector matches the order of clusters.
        grouped_cells.sort_by(|a, b| {
            a.cells()
                .iter()
                .sum::<usize>()
                .cmp(&b.cells().iter().sum::<usize>())
        });
        assert_eq!(grouped_cells.len(), clusters.len());
        for (i, cell_cluster) in grouped_cells.iter().enumerate() {
            let cells = cell_cluster.cells();
            assert_eq!(cells.len(), cells_per_cluster as usize);
            assert!(cells
                .iter()
                .all(|cell| { *cell < (clusters[i] + 1) * cells_per_cluster }))
        }
    }

    #[test]
    fn test_cluster_relative_cluster_size() {
        let cells = HashSet::from_iter([0usize, 1, 2, 4]);
        let total_cell_count = 10;
        let cluster_id = 42;
        let cluster = Cluster::new(cluster_id, cells.clone(), total_cell_count);
        assert_eq!(cluster.cluster_id(), cluster_id);
        assert_eq!(cluster.total_cell_count(), total_cell_count);
        assert_eq!(cells.len(), cluster.cells().len());
        assert_eq!(cells.len(), cells.intersection(cluster.cells()).count());
        assert_ulps_eq!(0.4, cluster.relative_cluster_size());
    }

    #[test]
    fn test_cluster_best_parent() {
        let parent_clusters: Vec<Cluster> = [
            vec![8, 9, 10, 11, 12, 13],
            vec![0usize, 1, 4],
            vec![2, 5, 6, 7],
        ]
        .into_iter()
        .enumerate()
        .map(|(i, cells)| Cluster::new(i, HashSet::from_iter(cells), 13))
        .collect();
        let empty_parents: Vec<Cluster> = Vec::new();
        let cells = HashSet::from_iter([0usize, 1, 2, 4]);
        let total_cell_count = 10;
        let cluster_id = 42;
        let cluster = Cluster::new(cluster_id, cells.clone(), total_cell_count);

        assert_eq!(cluster.best_parent(&parent_clusters), Ok(1));
        assert!(cluster.best_parent(&empty_parents).is_err());
    }
}
