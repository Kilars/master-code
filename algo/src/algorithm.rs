use crate::rest::ReferenceList;
use crate::spatial_filter::PointWithIndexReference;

use rstar::RTree;
use serde::{de, Deserialize};

#[derive(Deserialize, Clone)]
struct CsvTrajectory {
    #[serde(deserialize_with = "deserialize_json_string")]
    polyline: Vec<(f32, f32)>,
}

fn deserialize_json_string<'de, T: Deserialize<'de>, D: de::Deserializer<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    serde_json::from_str(Deserialize::deserialize(deserializer)?).map_err(de::Error::custom)
}
#[derive(Debug)]
pub struct Config {
    pub n: i32,
    pub compression_ratio: i32,
    pub spatial_filter: bool,
    pub error_trajectories: i32,
    pub error_point: i32,
}
pub fn rest_main(conf: Config) -> Result<(), csv::Error> {
    let par_records = csv::Reader::from_path("porto.csv")?
        .deserialize()
        .take(conf.n as usize)
        .map(|res| {
            res.map(|traj: CsvTrajectory| {
                traj.polyline
                    .iter()
                    .map(|&pnt| pnt.into())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut mrt_list = ReferenceList {
        trajectories: Vec::new(),
    };
    let mut r_tree: Option<RTree<PointWithIndexReference>> = if conf.spatial_filter {
        Some(RTree::new())
    } else {
        None
    };

    par_records.into_iter().for_each(|t| {
        let (_encoded_t, compression_ratio) = mrt_list.encode(
            &t,
            conf.error_trajectories as f64,
            conf.error_point as f64,
            r_tree.as_ref(),
        );
        if compression_ratio < conf.compression_ratio as f64 {
            if let Some(mut_tree) = r_tree.as_mut() {
                for (i, point) in t.iter().enumerate() {
                    mut_tree.insert(PointWithIndexReference {
                        point: point.clone(),
                        index: (mrt_list.trajectories.len(), i),
                    });
                }
            }
            mrt_list.trajectories.push(t);
        }
    });

    Ok(())
}
