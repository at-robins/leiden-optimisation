//! This module provides algorithms to calculate cluster stability.

use std::{borrow::Borrow, collections::HashSet};

/// Returns the number of cells overlapping between the 2 clusters.
///
/// # Parameters
///
/// * `cluster_a` - the first cluster to compare
/// * `cluster_b` - the second cluster to compare
pub fn cluster_overlap_absolute<A: Borrow<HashSet<usize>>, B: Borrow<HashSet<usize>>>(
    cluster_a: A,
    cluster_b: B,
) -> usize {
    cluster_a.borrow().intersection(cluster_b.borrow()).count()
}

/// Returns the relative overlap of the child cluster with the parent cluster.
/// Returns an error if the child cluster is empty.
///
/// # Parameters
///
/// * `cluster_parent` - the parent cluster to use as reference
/// * `cluster_child` - the child cluster calculate the stability from
pub fn cluster_overlap_relative<A: Borrow<HashSet<usize>>, B: Borrow<HashSet<usize>>>(
    cluster_parent: A,
    cluster_child: B,
) -> Result<f64, String> {
    if cluster_child.borrow().is_empty() {
        Err("The child cluster is empty.".to_string())
    } else {
        let overlap =
            cluster_overlap_absolute(cluster_parent.borrow(), cluster_child.borrow()) as f64;
        Ok((overlap as f64) / (cluster_child.borrow().len() as f64))
    }
}

/// Returns the relative overlaps of the child cluster with any of to the parent clusters.
/// Returns an error if the child cluster is empty.
///
/// # Parameters
///
/// * `cluster_parent` - the parent cluster to use as reference
/// * `cluster_child` - the child cluster calculate the stability from
pub fn cluster_overlaps_relative<A: Borrow<HashSet<usize>>, B: Borrow<HashSet<usize>>>(
    clusters_parent: &[A],
    cluster_child: B,
) -> Result<Vec<f64>, String> {
    if cluster_child.borrow().is_empty() {
        Err("The child cluster is empty.".to_string())
    } else {
        let overlaps = clusters_parent
            .iter()
            .map(|cluster_parent| {
                cluster_overlap_relative(cluster_parent.borrow(), cluster_child.borrow())
                    .expect("The child cluster cannot be empty as this has been verified before.")
            })
            .collect();
        Ok(overlaps)
    }
}

/// Returns the stability of the child cluster compared to the parent clusters.
/// Returns an error if the child cluster is empty.
/// The stability is defined as the sum of the squared relative overlaps.
///
/// # Parameters
///
/// * `cluster_parent` - the parent cluster to use as reference
/// * `cluster_child` - the child cluster calculate the stability from
pub fn cluster_stability<A: Borrow<HashSet<usize>>, B: Borrow<HashSet<usize>>>(
    clusters_parent: &[A],
    cluster_child: B,
) -> Result<f64, String> {
    let relative_overlaps = cluster_overlaps_relative(clusters_parent, cluster_child)?;
    Ok(relative_overlaps
        .into_iter()
        .map(|overlap| overlap.powi(2))
        .sum())
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use super::*;

    #[test]
    fn test_cluster_overlap_absolute_partial() {
        let set_a: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 2, 3, 7]);
        let set_b: HashSet<usize> = HashSet::from_iter(vec![0usize, 3, 9, 24, 42, 84, 182881821]);
        // Overlap with each other.
        assert_eq!(2, cluster_overlap_absolute(&set_a, &set_b));
        assert_eq!(2, cluster_overlap_absolute(&set_b, &set_a));
        // Overlap with self.
        assert_eq!(set_a.len(), cluster_overlap_absolute(&set_a, &set_a));
        assert_eq!(set_b.len(), cluster_overlap_absolute(&set_b, &set_b));
    }

    #[test]
    fn test_cluster_overlap_full() {
        let set_a: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 2, 3, 7]);
        let set_b: HashSet<usize> = HashSet::from_iter(vec![7usize, 1, 3, 0, 2]);
        assert_eq!(set_a.len(), set_b.len());
        // Overlap with each other.
        assert_eq!(set_a.len(), cluster_overlap_absolute(&set_a, &set_b));
        assert_eq!(set_a.len(), cluster_overlap_absolute(&set_b, &set_a));
        // Overlap with self.
        assert_eq!(set_a.len(), cluster_overlap_absolute(&set_a, &set_a));
        assert_eq!(set_a.len(), cluster_overlap_absolute(&set_b, &set_b));
    }

    #[test]
    fn test_cluster_overlap_none() {
        let set_a: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 2, 3, 7]);
        let set_b: HashSet<usize> = HashSet::from_iter(vec![8usize, 9, 10, 11, 42]);
        assert_eq!(0, cluster_overlap_absolute(&set_a, &set_b));
        assert_eq!(0, cluster_overlap_absolute(&set_b, &set_a));
    }

    #[test]
    fn test_cluster_overlap_empty() {
        let set_full: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 2, 3, 7]);
        let set_empty: HashSet<usize> = HashSet::new();
        assert_eq!(0, cluster_overlap_absolute(&set_empty, &set_empty));
        assert_eq!(0, cluster_overlap_absolute(&set_full, &set_empty));
        assert_eq!(0, cluster_overlap_absolute(&set_empty, &set_full));
    }

    #[test]
    fn test_cluster_overlap_relative_empty_child_cluster() {
        let cluster_parent: HashSet<usize> = HashSet::from_iter(vec![0usize, 1]);
        let cluster_child: HashSet<usize> = HashSet::new();
        // Overlap with each other.
        assert!(cluster_overlap_relative(&cluster_parent, &cluster_child).is_err());
    }

    #[test]
    fn test_cluster_overlap_relative_larger_child_cluster() {
        let cluster_parent: HashSet<usize> = HashSet::from_iter(vec![0usize, 1]);
        let cluster_child: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 3, 4, 5, 6, 10, 11]);
        assert_ulps_eq!(0.25, cluster_overlap_relative(&cluster_parent, &cluster_child).unwrap());
    }

    #[test]
    fn test_cluster_overlap_relative_larger_parent_cluster() {
        let cluster_parent: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 3, 4, 5, 6, 10, 11]);
        let cluster_child_full: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 4]);
        let cluster_child_partial: HashSet<usize> = HashSet::from_iter(vec![0usize, 12, 13, 14, 15]);
        let cluster_child_none: HashSet<usize> = HashSet::from_iter(vec![12, 13, 14, 15]);
        assert_ulps_eq!(
            1.0,
            cluster_overlap_relative(&cluster_parent, &cluster_child_full).unwrap()
        );
        assert_ulps_eq!(
            0.2,
            cluster_overlap_relative(&cluster_parent, &cluster_child_partial).unwrap()
        );
        assert_ulps_eq!(
            0.0,
            cluster_overlap_relative(&cluster_parent, &cluster_child_none).unwrap()
        );
    }

    #[test]
    fn test_cluster_overlaps_relative() {
        let clusters_parent: Vec<HashSet<usize>> = vec![
            HashSet::from_iter(vec![0usize, 3, 6]),
            HashSet::from_iter(vec![1usize, 4, 7]),
            HashSet::from_iter(vec![2usize, 5, 8]),
        ];
        let cluster_child: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 4, 7]);
        let expected_overlaps: Vec<f64> = vec![0.25, 0.75, 0.0];
        let observed_overlaps = cluster_overlaps_relative(&clusters_parent, cluster_child).unwrap();

        for (i, expected_overlap) in expected_overlaps.into_iter().enumerate() {
            assert_ulps_eq!(expected_overlap, observed_overlaps[i]);
        }
    }

    #[test]
    fn test_cluster_overlaps_empty() {
        let clusters_parent: Vec<HashSet<usize>> = vec![
            HashSet::from_iter(vec![0usize, 3, 6]),
            HashSet::from_iter(vec![1usize, 4, 7]),
            HashSet::from_iter(vec![2usize, 5, 8]),
        ];
        let cluster_child: HashSet<usize> = HashSet::new();
        assert!(cluster_overlaps_relative(&clusters_parent, cluster_child).is_err());
    }


    #[test]
    fn test_cluster_stability() {
        let clusters_parent: Vec<HashSet<usize>> = vec![
            HashSet::from_iter(vec![0usize, 3, 6]),
            HashSet::from_iter(vec![1usize, 4, 7]),
            HashSet::from_iter(vec![2usize, 5, 8]),
        ];
        let cluster_child: HashSet<usize> = HashSet::from_iter(vec![0usize, 1, 4, 7]);
        assert_ulps_eq!(0.625, cluster_stability(&clusters_parent, cluster_child).unwrap());
    }

    #[test]
    fn test_cluster_stability_child_empty() {
        let clusters_parent: Vec<HashSet<usize>> = vec![
            HashSet::from_iter(vec![0usize, 3, 6]),
            HashSet::from_iter(vec![1usize, 4, 7]),
            HashSet::from_iter(vec![2usize, 5, 8]),
        ];
        let cluster_child: HashSet<usize> = HashSet::new();
        assert!(cluster_stability(&clusters_parent, cluster_child).is_err());
    }
}
