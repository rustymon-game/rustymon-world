use crate::geometry::Point;

/// Combine an outer ring with its inner rings into a single polygon
pub fn combine_rings(outer_ring: &mut Vec<Point>, inner_rings: &mut [Vec<Point>]) {
    let mut temp = Vec::new();

    // Find the points in the inner rings whose x coordinate is lowest
    let mut inner_indexes = Vec::with_capacity(inner_rings.len());
    for inner_ring in inner_rings.iter() {
        inner_indexes.push(
            inner_ring
                .iter()
                .enumerate()
                .min_by(|(_, &a), (_, &b)| a.x.partial_cmp(&b.x).unwrap())
                .map(|(index, _)| index)
                .unwrap_or(0),
        );
    }

    // Rotate the inner rings to make their selected point the first in memory
    for (i, inner_ring) in inner_rings.iter_mut().enumerate() {
        let index = inner_indexes[i];
        temp.clear();
        temp.extend_from_slice(&inner_ring[index..]);
        temp.extend_from_slice(&inner_ring[0..index]);
        inner_ring.clone_from(&temp);
    }

    // Sort inner rings by their point with lowest x
    inner_rings.sort_by(|ring1, ring2| {
        let coord1 = ring1.first().map_or(f64::INFINITY, |point| point.x);
        let coord2 = ring2.first().map_or(f64::INFINITY, |point| point.x);
        coord1
            .partial_cmp(&coord2)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Merge the inner rings into the outer ring
    for inner_ring in inner_rings.iter_mut() {
        // Take any point from the inner ring
        let inner_point = if let Some(&point) = inner_ring.first() {
            point
        } else {
            // Skip empty inner ring
            continue;
        };

        // Find the point on the outer ring which is closest to the arbitrary inner one
        let index = outer_ring
            .iter()
            .enumerate()
            .filter(|(_, point)| point.x <= inner_point.x)
            .map(|(index, point)| (index, (point - inner_point).norm_squared()))
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(dist_b).unwrap())
            .map(|(index, _)| index);
        let index = if let Some(index) = index {
            index
        } else {
            // Skip if:
            // - outer ring is empty (i.e. iter() didn't yield anything)
            // - the inner ring's most left point lies left of the outer ring's ones
            continue;
        };

        // Add the inner ring starting at the inner point to the outer ring after the outer point
        // and add the outer point again
        inner_ring.push(inner_point);
        inner_ring.extend_from_slice(&outer_ring[index..]);
        outer_ring.truncate(index + 1);
        outer_ring.extend_from_slice(&inner_ring);
    }
}
