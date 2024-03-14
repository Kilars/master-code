use crate::rest::Point;
use rstar::{PointDistance, RTreeObject, AABB};

#[derive(Clone, PartialEq, Debug)]
pub struct PointWithIndexReference {
    pub point: Point,
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
