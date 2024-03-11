use crate::rest::{Point, ReferenceList};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

#[derive(Clone, PartialEq, Debug)]
pub struct PointWithIndexReference {
    point: Point,
    pub index: (usize, usize),
}

impl RTreeObject for PointWithIndexReference {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.point.lat_as_f32(), self.point.lng_as_f32()])
    }
}
impl PointDistance for PointWithIndexReference {
    fn distance_2(&self, point: &[f32; 2]) -> f32 {
        self.point.distance(&Point::from((point[0], point[1]))) as f32
    }
}

pub fn sequential_mrt_build_spatial_filter(ts: Vec<Vec<Point>>) -> ReferenceList {
    let mut mrt_list = ReferenceList {
        trajectories: Vec::new(),
    };
    let mut r_tree = RTree::<PointWithIndexReference>::new();
    let mut encode_count = 0;
    let trajectories = ts.len();
    ts.into_iter().for_each(|t| {
        println!("Encoded {} out of {}", encode_count, trajectories);
        println!(
            "Percentage done: {:.2}%",
            (encode_count as f32 * 100 as f32) / trajectories as f32
        );
        encode_count += 1;
        let (_encoded_t, compression_ratio) = mrt_list.encode(&t, 0.2, Some(&r_tree));
        println!("Compression ratio: {}", compression_ratio);
        if compression_ratio < 5.0 {
            for (i, point) in t.iter().enumerate() {
                r_tree.insert(PointWithIndexReference {
                    point: point.clone(),
                    index: (mrt_list.trajectories.len(), i),
                });
            }
            mrt_list.trajectories.push(t);
        }
    });
    mrt_list
}
