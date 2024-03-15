use std::{borrow::Borrow, collections::HashMap, rc::Rc};

use getset::{CopyGetters, Getters};

use crate::data::{ClusterStabilityData, ResolutionData};

/// Aggregates the [`ResolutionData`] vector by number of clusters present.
///
/// # Parameters
///
/// * `resplutions` - the resolution data to aggregate
pub fn aggregate_by_number_of_clusters(
    resolutions: &[ResolutionData],
) -> HashMap<usize, Vec<&ResolutionData>> {
    let mut cluster_map: HashMap<usize, Vec<&ResolutionData>> = HashMap::new();
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

/// Returns the root nodes of a cluster stability graph sampled at different resolutions
/// and ordered in layers depending on the respective number of clusters.
///
/// # Parameters
///
/// * `resolutions` - the resolution data to build the graph from
pub fn to_graph(resolutions: &[ResolutionData]) -> Vec<Rc<ResolutionNode>> {
    let map = aggregate_by_number_of_clusters(resolutions);
    let mut ordered_cluster_keys: Vec<usize> = map.keys().cloned().collect();
    ordered_cluster_keys.sort();

    // Returns an empty vector if there are no clusters.
    if ordered_cluster_keys.is_empty() {
        return Vec::new();
    }

    let mut potential_parent_nodes: Vec<Rc<ResolutionNode>> = Vec::new();
    let mut previous_cluster_key: Option<usize> = None;
    for (i, cluster_key) in ordered_cluster_keys.into_iter().enumerate() {
        let resolutions = map.get(&cluster_key).expect(
            "The key was obtained directly from the map so there must be an associated value.",
        );
        potential_parent_nodes = if i == 0 {
            // The first cluster elements do not have parent nodes.
            resolutions
                .iter()
                .map(|resolution| {
                    Rc::new(ResolutionNode::new(resolution.resolution(), resolution.clusters()))
                })
                .collect()
        } else {
            // Other cluster elements have parents and according stabilities.
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
                    let mut optimal_node: Option<ResolutionNode> = None;
                    for (i, potential_parent_node) in potential_parent_nodes.iter().enumerate() {
                        let stability_data = ClusterStabilityData::from_clustering(
                            resolution,
                            &previous_resolutions[i],
                        )
                        .expect(
                            "The number of clusters cannot be equal as sorting happend beforehand.",
                        );
                        let potential_child_node = ResolutionNode::new_with_parent(
                            resolution.resolution(),
                            resolution.clusters(),
                            potential_parent_node,
                            stability_data.mean_stability(),
                        );
                        // The optimal node has the highest overall stability and resolution.
                        // Defaults to true if unset so that the optimal node gets set on the first iteration.
                        if optimal_node.as_ref().map_or(true, |current_optimal_node| {
                            potential_child_node.total_stability()
                                > current_optimal_node.total_stability()
                                || (potential_child_node.total_stability()
                                    == current_optimal_node.total_stability())
                                    && potential_child_node.resolution()
                                        > current_optimal_node.resolution()
                        }) {
                            optimal_node = Some(potential_child_node)
                        }
                    }
                    Rc::new(optimal_node.expect(
                        "This must be set as there cannot be empty parent clustering data.",
                    ))
                })
                .collect()
        };
        previous_cluster_key = Some(cluster_key);
    }
    // Returns the optimal leaf nodes.
    potential_parent_nodes
}

#[derive(CopyGetters, Getters, Debug, PartialEq, PartialOrd, Clone)]
/// A node in a connected resolution graph, where edges are defined as cluster stability between nodes.
pub struct ResolutionNode {
    /// The resolution of the node if applicable.
    #[getset(get_copy = "pub")]
    resolution: f64,
    /// The number of clusters present at the node's resolution.
    #[getset(get_copy = "pub")]
    number_of_clusters: usize,
    /// The optimal parent node and the according cluster stability.
    #[getset(get = "pub")]
    // The data is organised in layers so there will be no cycles,
    // thus using a simple Rc is not producing memory leaks.
    optimal_parent: Option<Rc<Self>>,
    /// The cluster stability of the optimal parent-child-transition.
    #[getset(get_copy = "pub")]
    optimal_stability: Option<f64>,
    /// The sum of optimal stability transitions needed to reach this node from the root node.
    #[getset(get_copy = "pub")]
    total_stability: f64,
    /// The number of transitions needed to reach this node.
    #[getset(get_copy = "pub")]
    depth: usize,
}

impl ResolutionNode {
    /// Creates a new root node.
    ///
    /// # Parameters
    ///
    /// * `resolution` - the resolution of the node
    /// * `number_of_clusters` - the number of clusters present at the specified resolution
    pub fn new(resolution: f64, number_of_clusters: usize) -> Self {
        Self {
            resolution,
            number_of_clusters,
            optimal_parent: None,
            optimal_stability: None,
            total_stability: 0.0,
            depth: 0,
        }
    }

    /// Creates a new child node.
    ///
    /// # Parameters
    ///
    /// * `resolution` - the resolution of the node
    /// * `number_of_clusters` - the number of clusters present at the specified resolution
    /// * `optimal_parent` - stability-wise the optimal parent node for this child node
    /// * `optimal_stability` - the cluster stability of the optimal parent-child-transition.
    /// including the stability for the transition of parent to child
    pub fn new_with_parent<T: Borrow<Rc<Self>>>(
        resolution: f64,
        number_of_clusters: usize,
        optimal_parent: T,
        optimal_stability: f64,
    ) -> Self {
        let optimal_parent = Rc::clone(optimal_parent.borrow());
        let total_stability = optimal_parent.total_stability() + optimal_stability;
        let depth = optimal_parent.depth() + 1;
        Self {
            resolution,
            number_of_clusters,
            optimal_parent: Some(optimal_parent),
            optimal_stability: Some(optimal_stability),
            total_stability,
            depth,
        }
    }

    /// Returns the mean total stability after reaching this node.
    /// 
    /// If applied to the root node this will return `1.0` as there is 
    /// no stability transition associated with it.
    pub fn mean_total_stability(&self) -> f64 {
        if self.depth() < 1 {
            1.0
        } else {
            self.total_stability() / (self.depth() as f64)
        }
    }

    /// Returns the branch leading to the specified node, starting with the specified node 
    /// and tracing back to a root node.
    /// 
    /// # Parameters
    /// 
    /// * `node` - the node to which to compute the branch for
    pub fn branch<T: Borrow<Rc<Self>>>(node: T) -> Vec<Rc<Self>> {
        let mut branch = Vec::new();
        let mut current_node = Some(Rc::clone(node.borrow()));
        while current_node.is_some() {
            // This unwrap must work, as the option containing a value has just been checked.
            branch.push(Rc::clone(current_node.as_ref().unwrap()));
            current_node = current_node.and_then(|n| n.optimal_parent().clone());
        }
        branch
    }
}
