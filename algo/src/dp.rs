use crate::max_dtw::max_dtw_no_memo;
use crate::rest::Point;
use itertools::Itertools;

fn douglas_peucker(polyline: &[Point], epsilon: f64) -> Vec<Point> {
    let mut dp: Vec<usize> = vec![0, polyline.len() - 1];

    while true {
        let dp_vec = dp.iter().map(|&i| polyline[i].clone()).collect_vec();
        if max_dtw_no_memo(dp_vec.as_slice(), polyline) > epsilon {
            let mut max_dist = (0.0, None);
            for i in 0..dp.len() - 1 {
                for j in dp[i] + 1..dp[i + 1] {
                    let dist = perpendicular_distance(
                        &polyline[j],
                        &polyline[dp[i]],
                        &polyline[dp[i + 1]],
                    );
                    if dist > max_dist.0 {
                        max_dist = (dist, Some(j));
                    }
                }
            }
            match max_dist.1 {
                Some(j) => {
                    dp.push(j);
                    dp.sort();
                }
                None => break,
            }
        } else {
            break;
        }
    }
    dp.iter().map(|&i| polyline[i].clone()).collect_vec()
}

fn perpendicular_distance(p: &Point, p1: &Point, p2: &Point) -> f64 {
    // Constants
    let earth_radius = 6371000.0; // in meters

    // Convert degrees to radians
    let lat1 = p1.lat_as_f32().to_radians();
    let lng1 = p1.lng_as_f32().to_radians();
    let lat2 = p2.lat_as_f32().to_radians();
    let lng2 = p2.lng_as_f32().to_radians();
    let latc = p.lat_as_f32().to_radians();
    let lngc = p.lng_as_f32().to_radians();

    // Reference latitude for projection
    let lat_ref = (lat1 + lat2) / 2.0;

    // Convert geographic coordinates to Cartesian coordinates
    let x1 = earth_radius * (lng1 - lng1) * lat_ref.cos();
    let y1 = earth_radius * (lat1 - lat_ref);
    let x2 = earth_radius * (lng2 - lng1) * lat_ref.cos();
    let y2 = earth_radius * (lat2 - lat_ref);
    let xc = earth_radius * (lngc - lng1) * lat_ref.cos();
    let yc = earth_radius * (latc - lat_ref);

    // Compute the direction vector for line AB
    let dx = x2 - x1;
    let dy = y2 - y1;

    // Find intersection point D using projection formula
    let k = ((yc - y1) * dy + (xc - x1) * dx) / (dx.powi(2) + dy.powi(2));
    let xd = x1 + k * dx;
    let yd = y1 + k * dy;

    // Convert Cartesian back to geographic coordinates
    let lat_d = yd / earth_radius + lat_ref;
    let lng_d = xd / (earth_radius * lat_ref.cos()) + lng1;

    p.distance(&Point::from((lat_d.to_degrees(), lng_d.to_degrees())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_douglas_peucker_simple() {
        // Define a simple polyline (a square shape)
        let points = vec![
            Point::from((41.145700, -86.14900)),
            Point::from((41.145900, -86.14700)),
            Point::from((42.140300, -87.09500)),
            Point::from((41.145500, -86.10300)),
            Point::from((41.142800, -86.11000)),
            Point::from((41.140300, -86.09500)),
        ];

        // Epsilon is chosen such that the simplified polyline should ideally remove the middle points
        let epsilon = 0.2; // Adjust this based on the scale of your Point::distance implementation

        let simplified_polyline = douglas_peucker(&points, epsilon);

        // Ensure the simplified polyline has fewer points
        assert!(simplified_polyline.len() < points.len());

        // Ensure that no point in the original polyline deviates from the simplified version by more than epsilon
        let simplified_distances = points
            .iter()
            .map(|p| {
                simplified_polyline
                    .iter()
                    .map(|sp| p.distance(sp))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect::<Vec<f64>>();

        for dist in simplified_distances {
            assert!(dist <= epsilon, "Point deviates by more than epsilon");
        }

        println!("{:?}", simplified_polyline);

        // Optionally, check that the first and last points remain unchanged (common in simplification algorithms)
        assert_eq!(simplified_polyline[0], points[0]);
        assert_eq!(simplified_polyline.last(), points.last());
    }
}
