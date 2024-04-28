use crate::rest::Point;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

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
pub trait SpatialQuery {
    fn points_within_envelope(
        &self,
        envelope_size: f64,
        center_point: Point,
    ) -> Vec<&PointWithIndexReference>;
}
impl SpatialQuery for RTree<PointWithIndexReference> {
    fn points_within_envelope(
        &self,
        envelope_size: f64,
        center_point: Point,
    ) -> Vec<&PointWithIndexReference> {
        let lat_d = (envelope_size / 111319.9) as f32;
        let lng_d =
            envelope_size as f32 / (111319.9 * center_point.lat_as_f32().to_radians().cos());
        self.locate_in_envelope(&AABB::from_corners(
            [
                center_point.lat_as_f32() + lat_d as f32,
                center_point.lng_as_f32() + lng_d as f32,
            ],
            [
                center_point.lat_as_f32() - lat_d as f32,
                center_point.lng_as_f32() - lng_d as f32,
            ],
        ))
        .collect::<Vec<_>>()
    }
}
