use itertools::Itertools;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

#[allow(dead_code, unused_imports)]
#[path = "../generated/trajectory_generated.rs"]
mod trajectory_generated;
use trajectory_generated::trajectory::{Point, Trajectory, TrajectoryArgs};

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_polyline")]
    polyline: Vec<[f32; 2]>,
}

fn deserialize_polyline<'de, D>(deserializer: D) -> Result<Vec<[f32; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    struct PolylineVisitor;

    impl<'de> Visitor<'de> for PolylineVisitor {
        type Value = Vec<[f32; 2]>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string representing a polyline")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let polyline: Vec<[f32; 2]> = serde_json::from_str(value).map_err(de::Error::custom)?;
            Ok(polyline)
        }
    }

    deserializer.deserialize_str(PolylineVisitor)
}

fn main() -> Result<(), csv::Error> {
    let csv_trajectories: Vec<CsvTrajectory> = csv::Reader::from_path("sample.csv")?
        .deserialize::<CsvTrajectory>()
        .try_collect()?;

    let buffers: Vec<_> = csv_trajectories
        .into_iter()
        .map(|csv_trajectory| {
            let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);

            let points: Vec<_> = csv_trajectory
                .polyline
                .into_iter()
                .map(|[lat, lng]| Point::new(lat, lng))
                .collect();

            let args = TrajectoryArgs {
                id: Some(builder.create_string(&csv_trajectory.id)),
                polyline: Some(builder.create_vector(&points)),
            };

            let trajectory = Trajectory::create(&mut builder, &args);

            builder.finish(trajectory, None);
            builder.finished_data().to_vec()
        })
        .collect();

    Ok(())
}
