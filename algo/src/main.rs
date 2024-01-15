use std::{fs::File};
use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

struct Point {
    lat: f32,
    lng: f32,
}
struct Trajectory {
    id: String,
    polyline: Vec<Point>,
}
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
impl CsvTrajectory {
    fn to_trajectory(self) -> Trajectory {
        Trajectory {
            id: self.id,
            polyline: self.polyline.into_iter().map(|point| Point {
                lat: point[0],
                lng: point[1],
            }).collect(),
        }
    }
}

fn read_csv() -> Result<Vec<CsvTrajectory>, csv::Error> {
    let file = File::open("./sample.csv");
    let mut trajectories = Vec::new();
    match file {
        Ok(reader) => {
            let mut csv_reader = csv::Reader::from_reader(reader);
            for record in csv_reader.deserialize() {
                let record: CsvTrajectory = record?;
                trajectories.push(record);
            }
        }
        Err(err) => println!("Error reading file, {err}"),
    }
    Ok(trajectories)
}

fn main() {
    match read_csv() {
        Ok(csv_trajectories) => {
            for csv_traj in csv_trajectories {
                let traj = csv_traj.to_trajectory();
                println!("Trajectory id: {}", traj.id);
                for point in traj.polyline {
                    println!("Point: {}, {}", point.lat, point.lng);
                }
            }
        }
        Err(err) => println!("Error reading file, {err}"),
    }
}
