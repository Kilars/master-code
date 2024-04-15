use crate::rest::{EncodedTrajectory, Point, SubTrajectory};
use plotters::prelude::*;

pub fn graph_trajectory(
    path: &str,
    ogpath: &str,
    encoded: EncodedTrajectory,
    original_trajectory: Vec<Point>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let backend = BitMapBackend::new(path, (600, 400)).into_drawing_area();
    backend.fill(&WHITE)?;

    // Calculate the bounds based on original_trajectory
    let (min_lat, max_lat, min_lng, max_lng) = original_trajectory.iter().fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_lat, max_lat, min_lng, max_lng), pnt| {
            (
                min_lat.min(pnt.lat),
                max_lat.max(pnt.lat),
                min_lng.min(pnt.lng),
                max_lng.max(pnt.lng),
            )
        },
    );

    // Adjust the bounds slightly to ensure all points are within view
    let margin = |max: i32, min: i32| (max - min) / 10;
    let lat_margin = margin(max_lat, min_lat);
    let lng_margin = margin(max_lng, min_lng);
    let km_lat = 4.5 * 200.0;

    let mut ctx = ChartBuilder::on(&backend)
        .set_all_label_area_size(40)
        .build_cartesian_2d(
            min_lat - lat_margin..max_lat + lat_margin,
            min_lng - lng_margin..max_lng + lng_margin,
        )
        .unwrap();

    ctx.configure_mesh()
        .disable_mesh()
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    let mut last_point = Point { lat: 0, lng: 0 };
    for st in encoded.0.iter() {
        match st {
            SubTrajectory::Reference(reference) => {
                let mut t = Vec::new();
                for point in reference.iter() {
                    t.push(point.clone());
                }
                if last_point.lat != 0 {
                    ctx.draw_series(LineSeries::new(
                        vec![(last_point.lat, last_point.lng), (t[0].lat, t[0].lng)],
                        &GREEN,
                    ))?;
                }
                ctx.draw_series(LineSeries::new(
                    t.iter().map(|pnt| (pnt.lat, pnt.lng)),
                    &RED,
                ))?;
                last_point = t[t.len() - 1].clone();
            }
            SubTrajectory::Trajectory(trajectory) => {
                let mut t = Vec::new();
                for point in trajectory.iter() {
                    t.push(point.clone());
                }
                if last_point.lat != 0 {
                    ctx.draw_series(LineSeries::new(
                        vec![(last_point.lat, last_point.lng), (t[0].lat, t[0].lng)],
                        &GREEN,
                    ))?;
                }
                ctx.draw_series(LineSeries::new(
                    t.iter().map(|pnt| (pnt.lat, pnt.lng)),
                    &BLUE,
                ))?;
                last_point = t[t.len() - 1].clone();
            }
        }
    }
    ctx.draw_series(LineSeries::new(
        vec![(max_lat, max_lng), (max_lat + km_lat as i32, max_lng)],
        &YELLOW,
    ))?
    .label("1km hellooo");

    let backend2 = BitMapBackend::new(ogpath, (600, 400)).into_drawing_area();
    backend2.fill(&WHITE)?;

    let og_as_tuples: Vec<(i32, i32)> = original_trajectory
        .iter()
        .map(|pnt| (pnt.lat, pnt.lng))
        .collect();

    let mut ctx2 = ChartBuilder::on(&backend2)
        .build_cartesian_2d(
            min_lat - lat_margin..max_lat + lat_margin,
            min_lng - lng_margin..max_lng + lng_margin,
        )
        .unwrap();

    ctx2.configure_mesh()
        .disable_mesh()
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    ctx2.draw_series(LineSeries::new(og_as_tuples, &BLUE))?;

    backend.present()?;
    backend2.present()?;

    Ok(())
}
