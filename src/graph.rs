use std::{collections::HashMap, hash::Hash, rc::Rc};

use getset::{CopyGetters, Getters};
use ordered_float::OrderedFloat;

use crate::data::{ClusterStabilityData, ResolutionData};

const FAKE_NODE_STABILITY: OrderedFloat<f64> = OrderedFloat(1.0);

/// Aggregates the [`ResolutionData`] vector by number of clusters present.
///
/// # Parameters
///
/// * `resplutions` - the resolution data to aggregate
pub fn aggregate_by_number_of_clusters(
    resolutions: Vec<ResolutionData>,
) -> HashMap<usize, Vec<ResolutionData>> {
    let mut cluster_map: HashMap<usize, Vec<ResolutionData>> = HashMap::new();
    for resolution in resolutions {
        let number_of_clusters = resolution.clusters();
        if let Some(cluster_data) = cluster_map.get_mut(&number_of_clusters) {
            cluster_data.push(resolution)
        } else {
            cluster_map.insert(number_of_clusters, vec![resolution]);
        }
    }
    cluster_map
}

/// Returns the root node of a cluster stability tree sampled at different resolutions.
///
/// # Parameters
///
/// * `resolutions` - the resolution data to build the tree from
pub fn to_tree(resolutions: Vec<ResolutionData>) -> Rc<ResolutionNode> {
    let map = aggregate_by_number_of_clusters(resolutions);
    let mut ordered_cluster_keys: Vec<usize> = map.keys().cloned().collect();
    ordered_cluster_keys.sort();
    // Starts with the final node that has a fake resolution, which is never used.
    let mut successor_nodes: Vec<Rc<ResolutionNode>> =
        vec![Rc::new(ResolutionNode::create_end_node())];
    let mut previous_cluster_key: Option<usize> = None;
    for (i, cluster_key) in ordered_cluster_keys.into_iter().rev().enumerate() {
        let resolutions = map.get(&cluster_key).expect(
            "The key was obtained directly from the map so there must be an associated value.",
        );
        successor_nodes = if i == 0 {
            // The first element / the highest cluster number.
            resolutions
                .iter()
                .map(|resolution| {
                    Rc::new(ResolutionNode::create_intermediate_node(
                        resolution.resolution(),
                        successor_nodes
                            .iter()
                            // The distance to the fake end node is set to the minimum distance value / maximum stability.
                            .map(|end_node| (Rc::clone(end_node), FAKE_NODE_STABILITY))
                            .collect(),
                    ))
                })
                .collect()
        } else {
            // Intermediate elements.
            let previous_resolutions = map.get(
                previous_cluster_key
                    .as_ref()
                    .expect("The previous key must have been set on a previous iteration."),
            )
            .expect(
                "The key was obtained directly from the map so there must be an associated value.",
            );
            resolutions
                .iter()
                .map(|resolution| {
                    Rc::new(ResolutionNode::create_intermediate_node(
                        resolution.resolution(),
                        successor_nodes
                        .iter()
                        .enumerate()
                        // The distance to the fake end node is set to the minimum value.
                            .map(|(i, node)| {
                                let stability_data = ClusterStabilityData::from_clustering(resolution, &previous_resolutions[i]).expect("The number of clusters cannot be equal as sorting happend beforehand.");
                                (Rc::clone(node), OrderedFloat(stability_data.mean_stability()))
                            })
                            .collect(),
                    ))
                })
                .collect()
        };
        previous_cluster_key = Some(cluster_key);
    }
    // Returns a fake starting node with minimal distance / maximal stability to all other nodes with the lowest cluster number.
    Rc::new(ResolutionNode::create_start_node(
        successor_nodes
            .iter()
            .map(|node| (Rc::clone(node), FAKE_NODE_STABILITY))
            .collect(),
    ))
}

/// Returns the path that optimally conserves stability between clusters in a tree of cluster [`ResolutionData`].
/// 
/// # Parameters
///
/// * `resolutions` - the resolution data to calulate the optimal stability path for
pub fn optimal_stability_path(resolutions: Vec<ResolutionData>) -> Option<(Vec<Rc<ResolutionNode>>, OrderedFloat<f64>)> {
    println!("Creating cluster stability tree.");
    let root = to_tree(resolutions);
    println!("Calculating shortest path.");
    pathfinding::directed::dijkstra::dijkstra(
        &root,
        |node| node.successors_with_distance(),
        |node| node.is_final_node(),
    )
}

#[derive(CopyGetters, Getters, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
/// A node in a connected resolution graph, where edges are defined as cluster stability between nodes.
pub struct ResolutionNode {
    /// The resolution of the node if applicable.
    /// Starting and end node do not have a resolution as they are fake nodes.
    #[getset(get_copy = "pub")]
    resolution: Option<OrderedFloat<f64>>,
    /// The nodes and stabilities connected to this node.
    #[getset(get = "pub")]
    // There will be no cycles so using a simple Rc is not producing memory leaks.
    successors: Vec<(Rc<ResolutionNode>, OrderedFloat<f64>)>,
}

impl ResolutionNode {
    /// Returns the successor nodes with the according distance, which is inversly proportional to the cluster stability between the two nodes.
    pub fn successors_with_distance(&self) -> Vec<(Rc<ResolutionNode>, OrderedFloat<f64>)> {
        self.successors
            .iter()
            .map(|(resolution, stability)| (Rc::clone(resolution), OrderedFloat(1.0) - stability))
            .collect()
    }

    /// Returns ```true``` if the node is an end / leaf node.
    pub fn is_final_node(&self) -> bool {
        self.successors.is_empty()
    }

    /// Creates an end / leaf node.
    fn create_end_node() -> Self {
        Self {
            resolution: None,
            successors: Vec::new(),
        }
    }

    /// Creates a start / root node with the specified successor nodes.
    ///
    /// # Parameters
    ///
    /// * `successors` - the connected child nodes
    fn create_start_node(successors: Vec<(Rc<ResolutionNode>, OrderedFloat<f64>)>) -> Self {
        Self {
            resolution: None,
            successors,
        }
    }

    /// Creates an intermediate node with the specified resolution and successor nodes.
    ///
    /// # Parameters
    ///
    /// * `resolution` - the resolution of the clustering the node corresponds to
    /// * `successors` - the connected child nodes
    fn create_intermediate_node(
        resolution: f64,
        successors: Vec<(Rc<ResolutionNode>, OrderedFloat<f64>)>,
    ) -> Self {
        Self {
            resolution: Some(resolution.into()),
            successors,
        }
    }
}
