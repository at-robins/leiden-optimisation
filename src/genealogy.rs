use std::{borrow::Borrow, collections::HashMap, rc::Rc};

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{
    data::{Cluster, ResolutionData},
    graph::ResolutionNode,
};

#[derive(CopyGetters, Getters, Clone, Debug, Deserialize, Serialize)]
/// A node in a cluster relation tree over different clustering resolutions.
pub struct ClusterGenealogyNode {
    cluster_id: usize,
    origin_resolution: f64,
    child_clusters: Vec<Self>,
}

impl ClusterGenealogyNode {
    /// Creates a new node with the specified cluster ID and origin resolution.
    ///
    /// # Parameters
    ///
    /// * `cluster_id` - the ID of the cluster that this node represents
    /// * `origin_resolution` - the resolution the cluster was observed at
    pub fn new(cluster_id: usize, origin_resolution: f64) -> Self {
        Self {
            cluster_id,
            origin_resolution,
            child_clusters: Vec::new(),
        }
    }

    /// Adds a child cluster to this parent cluster.
    ///
    /// # Parameters
    ///
    /// * `child_cluster` - the cluster node to set as child of this cluster
    pub fn add_child_cluster(&mut self, child_cluster: Self) {
        self.child_clusters.push(child_cluster);
    }

    /// Sorts the child nodes by cluster ID.
    pub fn sort(&mut self) {
        self.child_clusters
            .sort_by(|a, b| a.cluster_id.cmp(&b.cluster_id));
    }

    /// Builds a cluster relation tree from a set of resolutions.
    ///
    /// # Parameters
    ///
    /// * `child_cluster` - the cluster node to set as child of this cluster
    pub fn from_resolution_data<T: Borrow<ResolutionData>>(
        data: &[T],
    ) -> Result<Vec<ClusterGenealogyNode>, &'static str> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
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
            .map(|cluster| {
                (
                    ClusterGenealogyNode::new(cluster.cluster_id(), bottom_resolution.resolution()),
                    cluster,
                )
            })
            .collect();
        for top_resolution_index in ordered_iter {
            let top_resolution: &ResolutionData = data[top_resolution_index].borrow();
            let mut top_nodes: HashMap<usize, (ClusterGenealogyNode, &Cluster)> = top_resolution
                .clustered_cells()
                .iter()
                .map(|cluster| {
                    (
                        cluster.cluster_id(),
                        (
                            ClusterGenealogyNode::new(
                                cluster.cluster_id(),
                                top_resolution.resolution(),
                            ),
                            cluster,
                        ),
                    )
                })
                .collect();
            for (bottom_node, bottom_cluster) in bottom_nodes.into_iter() {
                let parent_id = bottom_cluster.best_parent(top_resolution.clustered_cells())?;
                match top_nodes.get_mut(&parent_id) {
                    Some(parent_node) => parent_node.0.add_child_cluster(bottom_node),
                    None => return Err("Parent node not found!"),
                }
            }
            bottom_nodes = top_nodes.into_values().collect();
            // Sorts the child nodes by cluster ID.
            bottom_nodes.iter_mut().for_each(|(node, _)| node.sort());
        }

        Ok(bottom_nodes.into_iter().map(|(node, _)| node).collect())
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
