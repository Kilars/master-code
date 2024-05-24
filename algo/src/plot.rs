use crate::rest::{EncodedTrajectory, Point, SubTrajectory};
use plotters::prelude::*;

pub fn graph_trajectory(
    path: String,
    encoded: EncodedTrajectory,
    original_trajectory: Vec<Point>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let backend = BitMapBackend::new(&path, (600, 400)).into_drawing_area();
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

    ctx.draw_series(LineSeries::new(
        original_trajectory.iter().map(|pnt| (pnt.lat, pnt.lng)),
        &BLACK,
    ));
    let mut last_pnt: Option<Point> = None;
    for st in encoded.0.iter() {
        match st {
            SubTrajectory::Reference(reference) => {
                let mut t = Vec::new();
                last_pnt.map(|pnt| t.push(pnt.clone()));
                for point in reference.iter() {
                    t.push(point.clone());
                }
                last_pnt = t.last().map(|pnt| pnt.clone());
                ctx.draw_series(LineSeries::new(
                    t.iter().map(|pnt| (pnt.lat, pnt.lng)),
                    &RED,
                ))?;
            }
            SubTrajectory::Trajectory(trajectory) => {
                let mut t = Vec::new();
                last_pnt.map(|pnt| t.push(pnt.clone()));
                for point in trajectory.iter() {
                    t.push(point.clone());
                }
                last_pnt = t.last().map(|pnt| pnt.clone());
                ctx.draw_series(LineSeries::new(
                    t.iter().map(|pnt| (pnt.lat, pnt.lng)),
                    &RED,
                ))?;
            }
        }
    }
    backend.present()?;

    Ok(())
}
