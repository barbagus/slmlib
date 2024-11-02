use crate::geo::Point;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct SMLPoint {
    #[serde(rename = "Latitude")]
    pub latitude: f64,
    #[serde(rename = "Longitude")]
    pub longitude: f64,
    #[serde(rename = "Order")]
    pub order: usize,
    #[serde(rename = "CtrlPtLat")]
    pub control_point_latitude: f64,
    #[serde(rename = "CtrlPtLng")]
    pub control_point_longitude: f64,
    #[serde(rename = "DistToLine")]
    pub distance_to_line: f64,
    #[serde(rename = "CtrlPtDistToStart")]
    pub control_point_distance_to_start: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SMLTargetPoint {
    #[serde(rename = "Latitude")]
    pub latitude: f64,
    #[serde(rename = "Longitude")]
    pub longitude: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SMLAttempt {
    #[serde(rename = "Points")]
    pub points: Vec<SMLPoint>,
    #[serde(rename = "TLStart")]
    pub target_line_start: Option<SMLTargetPoint>,
    #[serde(rename = "TLEnd")]
    pub target_line_end: Option<SMLTargetPoint>,
    #[serde(rename = "TargetLineLength")]
    pub target_line_length: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SMLDoc {
    #[serde(rename = "Attempt")]
    pub attempt: SMLAttempt,
    #[serde(rename = "Init_Drop")]
    pub init_drop: u32,
    #[serde(rename = "Init_ScoreLevel")]
    pub init_score_level: u32,
    #[serde(rename = "Name")]
    pub name: String,
}

impl SMLAttempt {
    pub fn target_line(&self) -> (Point, Point) {
        if let Some(ref start) = self.target_line_start {
            let end = self.target_line_end.as_ref().unwrap();
            (
                Point::new(start.latitude, start.longitude).unwrap(),
                Point::new(end.latitude, end.longitude).unwrap(),
            )
        } else {
            assert!(self.target_line_end.is_none());
            let start = self.points.first().unwrap();
            let end = self.points.last().unwrap();
            (
                Point::new(start.latitude, start.longitude).unwrap(),
                Point::new(end.latitude, end.longitude).unwrap(),
            )
        }
    }

    pub fn geo_points(&self) -> impl Iterator<Item = Point> + '_ {
        self.points
            .iter()
            .map(|p| Point::new(p.latitude, p.longitude).unwrap())
    }
}
