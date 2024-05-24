use crate::max_dtw::max_dtw_no_memo;
use crate::rest::Point;
use itertools::Itertools;

fn douglas_peucker(polyline: &[Point], epsilon: f64) -> Vec<Point> {
    let mut dp: Vec<usize> = vec![0, polyline.len() - 1];

    while true {
        let dp_vec = dp.iter().map(|&i| polyline[i].clone()).collect_vec();
        if max_dtw_no_memo(dp_vec.as_slice(), polyline) > epsilon {
            let (index, max_distance) = (1..polyline.len() - 1)
                .map(|i| {
                    (
                        i,
                        perpendicular_distance(
                            &polyline[i],
                            &polyline[0],
                            &polyline[polyline.len() - 1],
                        ),
                    )
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
        }
    }
    todo!()
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
