use std::{borrow::Borrow, collections::HashSet};

/// Returns the number of cells overlapping between the 2 clusters.
///
/// # Parameters
///
/// * `cluster_a` - the first cluster to compare
/// * `cluster_b` - the second cluster to compare
pub fn cluster_overlap<A: Borrow<HashSet<u64>>, B: Borrow<HashSet<u64>>>(
    cluster_a: A,
    cluster_b: B,
) -> usize {
    cluster_a.borrow().intersection(cluster_b.borrow()).count()
}

/// Returns the stability of the child cluster compared to the parent cluster.
/// Returns an error if the child clusters is empty.
///
/// # Parameters
///
/// * `cluster_parent` - the parent cluster to use as reference
/// * `cluster_child` - the child cluster calculate the stability from
pub fn cluster_stability<A: Borrow<HashSet<u64>>, B: Borrow<HashSet<u64>>>(
    cluster_parent: A,
    cluster_child: B,
) -> Result<f64, String> {
    if cluster_child.borrow().is_empty() {
        Err("The parent cluster is empty.".to_string())
    } else {
        let overlap = cluster_overlap(cluster_parent.borrow(), cluster_child.borrow()) as f64;
        Ok((overlap as f64) / (cluster_child.borrow().len() as f64))
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn test_cluster_overlap_partial() {
        let set_a: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 2, 3, 7]);
        let set_b: HashSet<u64> = HashSet::from_iter(vec![0u64, 3, 9, 24, 42, 84, 182881821]);
        // Overlap with each other.
        assert_eq!(2, cluster_overlap(&set_a, &set_b));
        assert_eq!(2, cluster_overlap(&set_b, &set_a));
        // Overlap with self.
        assert_eq!(set_a.len(), cluster_overlap(&set_a, &set_a));
        assert_eq!(set_b.len(), cluster_overlap(&set_b, &set_b));
    }

    #[test]
    fn test_cluster_overlap_full() {
        let set_a: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 2, 3, 7]);
        let set_b: HashSet<u64> = HashSet::from_iter(vec![7u64, 1, 3, 0, 2]);
        assert_eq!(set_a.len(), set_b.len());
        // Overlap with each other.
        assert_eq!(set_a.len(), cluster_overlap(&set_a, &set_b));
        assert_eq!(set_a.len(), cluster_overlap(&set_b, &set_a));
        // Overlap with self.
        assert_eq!(set_a.len(), cluster_overlap(&set_a, &set_a));
        assert_eq!(set_a.len(), cluster_overlap(&set_b, &set_b));
    }

    #[test]
    fn test_cluster_overlap_none() {
        let set_a: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 2, 3, 7]);
        let set_b: HashSet<u64> = HashSet::from_iter(vec![8u64, 9, 10, 11, 42]);
        assert_eq!(0, cluster_overlap(&set_a, &set_b));
        assert_eq!(0, cluster_overlap(&set_b, &set_a));
    }

    #[test]
    fn test_cluster_overlap_empty() {
        let set_full: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 2, 3, 7]);
        let set_empty: HashSet<u64> = HashSet::new();
        assert_eq!(0, cluster_overlap(&set_empty, &set_empty));
        assert_eq!(0, cluster_overlap(&set_full, &set_empty));
        assert_eq!(0, cluster_overlap(&set_empty, &set_full));
    }

    #[test]
    fn test_cluster_stability_empty_child_cluster() {
        let cluster_parent: HashSet<u64> = HashSet::from_iter(vec![0u64, 1]);
        let cluster_child: HashSet<u64> = HashSet::new();
        // Overlap with each other.
        assert!(cluster_stability(&cluster_parent, &cluster_child).is_err());
    }

    #[test]
    fn test_cluster_stability_larger_child_cluster() {
        let cluster_parent: HashSet<u64> = HashSet::from_iter(vec![0u64, 1]);
        let cluster_child: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 3, 4, 5, 6, 10, 11]);
        assert_ulps_eq!(0.25, cluster_stability(&cluster_parent, &cluster_child).unwrap());
    }

    #[test]
    fn test_cluster_stability_larger_parent_cluster() {
        let cluster_parent: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 3, 4, 5, 6, 10, 11]);
        let cluster_child_full: HashSet<u64> = HashSet::from_iter(vec![0u64, 1, 4]);
        let cluster_child_partial: HashSet<u64> = HashSet::from_iter(vec![0u64, 12, 13, 14, 15]);
        let cluster_child_none: HashSet<u64> = HashSet::from_iter(vec![12, 13, 14, 15]);        
        assert_ulps_eq!(1.0, cluster_stability(&cluster_parent, &cluster_child_full).unwrap());
        assert_ulps_eq!(0.2, cluster_stability(&cluster_parent, &cluster_child_partial).unwrap());
        assert_ulps_eq!(0.0, cluster_stability(&cluster_parent, &cluster_child_none).unwrap());
    }
}
