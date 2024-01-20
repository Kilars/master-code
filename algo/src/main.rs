use itertools::Itertools;
use serde::{de, Deserialize};

#[allow(dead_code, unused_imports)]
#[path = "../generated/trajectory_generated.rs"]
mod trajectory_generated;
use trajectory_generated::trajectory::{Point, Trajectory, TrajectoryArgs};

#[derive(Deserialize)]
struct CsvTrajectory {
    id: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<[f32; 2]>,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
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
