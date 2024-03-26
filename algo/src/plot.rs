use plotters::prelude::*;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 800*600 bitmap and start drawing
    let backend = BitMapBackend::new("plots/traj.png", (600, 400)).into_drawing_area();
    backend.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&backend)
        .build_cartesian_2d(0..100, 0..100)
        .unwrap();
    ctx.draw_series(LineSeries::new([(2, 5), (5, 7), (1, 1)], &RED))
        .unwrap();

    backend.present()?;

    Ok(())
}
