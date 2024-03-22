use std::{borrow::Borrow, collections::HashMap, rc::Rc};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{
    data::{Cluster, ResolutionData},
    graph::ResolutionNode,
    optimisation::ClusterStabilityRegression,
};

#[derive(CopyGetters, Getters, Clone, Debug, Deserialize, Serialize)]
/// A node in a cluster relation tree over different clustering resolutions.
pub struct ClusterGenealogyNode {
    #[getset(get_copy = "pub")]
    /// The ID of the cluster.
    cluster_id: usize,
    child_clusters: Vec<usize>,
}

#[derive(CopyGetters, Getters, Clone, Debug, Deserialize, Serialize)]
/// An entry containing cluster relation data of all clusters sampled at a
/// specific resolution.
pub struct ClusterGenealogyEntry {
    #[getset(get_copy = "pub")]
    /// The number of clusters present at this resolution.
    number_of_clusters: usize,
    resolution: f64,
    nodes: Vec<ClusterGenealogyNode>,
}

impl ClusterGenealogyEntry {
    /// Creates a new cluster relation entry containing cluster relation data of
    /// all clusters sampled at a specific resolution.
    ///
    /// # Parameters
    ///
    /// * `resolution_data` - the [`ResolutionData`] the clusters have been sampled at
    /// * `nodes` - the individual cluster nodes
    pub fn new(resolution_data: &ResolutionData, mut nodes: Vec<ClusterGenealogyNode>) -> Self {
        nodes.iter_mut().for_each(ClusterGenealogyNode::sort);
        nodes.sort_by(|a, b| a.cluster_id().cmp(&b.cluster_id()));
        Self {
            number_of_clusters: resolution_data.clusters(),
            resolution: resolution_data.resolution(),
            nodes,
        }
    }

    /// Builds a cluster relation tree from a set of resolutions.
    ///
    /// # Parameters
    ///
    /// * `child_cluster` - the cluster node to set as child of this cluster
    pub fn from_resolution_data<T: Borrow<ResolutionData>>(
        data: &[T],
    ) -> Result<Vec<ClusterGenealogyEntry>, &'static str> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::with_capacity(data.len());
        // The ordering is performed on the clusters not on the whole array, to prevent copying of large amounts of data.
        let mut resolution_data_ordering: Vec<(usize, usize)> = data
            .iter()
            .enumerate()
            .map(|(index, data)| (index, data.borrow().clusters()))
            .collect();
        // The resolution data is ordered by decreasing cluster number.
        resolution_data_ordering.sort_by(|(_, a), (_, b)| b.cmp(a));
        let mut ordered_iter = resolution_data_ordering.into_iter().map(|(index, _)| index);
        let bottom_resolution: &ResolutionData = data[ordered_iter
            .next()
            .expect("The iterator cannot be empty as this has been checked before.")]
        .borrow();
        let mut bottom_nodes: Vec<(ClusterGenealogyNode, &Cluster)> = bottom_resolution
            .clustered_cells()
            .iter()
            .map(|cluster| (ClusterGenealogyNode::new(cluster.cluster_id()), cluster))
            .collect();
        entries.push(ClusterGenealogyEntry::new(
            bottom_resolution,
            bottom_nodes.iter().map(|(node, _)| node.clone()).collect(),
        ));
        for top_resolution_index in ordered_iter {
            let top_resolution: &ResolutionData = data[top_resolution_index].borrow();
            let mut top_nodes: HashMap<usize, (ClusterGenealogyNode, &Cluster)> = top_resolution
                .clustered_cells()
                .iter()
                .map(|cluster| {
                    (
                        cluster.cluster_id(),
                        (ClusterGenealogyNode::new(cluster.cluster_id()), cluster),
                    )
                })
                .collect();
            for (bottom_node, bottom_cluster) in bottom_nodes.into_iter() {
                let parent_id = bottom_cluster.best_parent(top_resolution.clustered_cells())?;
                match top_nodes.get_mut(&parent_id) {
                    Some(parent_node) => parent_node.0.add_child_cluster(bottom_node.cluster_id),
                    None => return Err("Parent node not found!"),
                }
            }
            entries.push(ClusterGenealogyEntry::new(
                top_resolution,
                top_nodes.values().map(|(node, _)| node.clone()).collect(),
            ));
            bottom_nodes = top_nodes.into_values().collect();
        }
        entries.sort_by(|a, b| a.number_of_clusters().cmp(&b.number_of_clusters()));
        Ok(entries)
    }
}

impl ClusterGenealogyNode {
    /// Creates a new node with the specified cluster ID.
    ///
    /// # Parameters
    ///
    /// * `cluster_id` - the ID of the cluster that this node represents
    pub fn new(cluster_id: usize) -> Self {
        Self {
            cluster_id,
            child_clusters: Vec::new(),
        }
    }

    /// Adds a child cluster to this parent cluster.
    ///
    /// # Parameters
    ///
    /// * `child_cluster` - the ID of the cluster to set as child of this cluster
    pub fn add_child_cluster(&mut self, child_cluster_id: usize) {
        self.child_clusters.push(child_cluster_id);
    }

    /// Sorts the child nodes by cluster ID.
    pub fn sort(&mut self) {
        self.child_clusters.sort_by(|a, b| a.cmp(&b));
    }
}

/// Returns the according resolution data for a branch of cluster stability data or
/// an error if any of the branch resolutions is not found in the specified [`ResolutionData`]
/// pool.
///
/// # Parameters
///
/// * `branch` - the branch to get the resolution data for
/// * `resolutions` - the pool of all [`ResolutionData`]s
pub fn branch_to_resolution_data<'a, 'b>(
    branch: &'a [Rc<ResolutionNode>],
    resolutions: &'b [ResolutionData],
) -> Result<Vec<&'b ResolutionData>, &'static str> {
    let mut branch_resolution_data = Vec::new();
    for node in branch {
        branch_resolution_data.push(
            resolutions
                .iter()
                .find(|resolution| resolution.resolution() == node.resolution())
                .ok_or("The resolution pool does not contain all branch resolutions.")?,
        );
    }
    Ok(branch_resolution_data)
}

/// Removes all nodes from the branch that do not pass the specified stability threshold.
///
/// # Parameters
///
/// * `branch` - the branch to trim
/// * `threshold` - the stability threshold
pub fn trim_branch(branch: &[Rc<ResolutionNode>], threshold: f64) -> Vec<Rc<ResolutionNode>> {
    let regression = ClusterStabilityRegression::new(branch);
    let mut branch: Vec<Rc<ResolutionNode>> = branch.iter().map(Rc::clone).collect();
    branch.sort_by(|a, b| a.number_of_clusters().cmp(&b.number_of_clusters()));
    let mut trimmed_branch = Vec::new();
    for node in branch.into_iter() {
        if regression.predict(node.number_of_clusters() as f64) >= threshold {
            // Keep nodes that are above the stability threshold.
            trimmed_branch.push(node);
        } else {
            // When the threshold is crossed for the first time,
            // discard all future clusterings.
            break;
        }
    }

    trimmed_branch
}
