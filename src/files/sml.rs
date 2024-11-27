// Copyright 2024 Barbagus
//
// This file is part of slmlib.
//
// slmlib is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// slmlib is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
// You should have received a copy of the GNU General Public License along with slmlib. If not, see
// <https://www.gnu.org/licenses/>.

//! A JSON file used by <https://scoremyline.com/>. No official specification.

extern crate alloc;

use crate::Coordinates;
use alloc::vec::Vec;
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
struct SMLDoc {
    #[serde(rename = "Attempt")]
    attempt: SMLAttempt,
}

impl SMLAttempt {
    pub fn route(&self) -> (Coordinates, Coordinates) {
        if let Some(ref start) = self.target_line_start {
            let end = self.target_line_end.as_ref().unwrap();
            (
                Coordinates {
                    latitude: start.latitude,
                    longitude: start.longitude,
                },
                Coordinates {
                    latitude: end.latitude,
                    longitude: end.longitude,
                },
            )
        } else {
            assert!(self.target_line_end.is_none());
            let start = self.points.first().unwrap();
            let end = self.points.last().unwrap();
            (
                Coordinates {
                    latitude: start.latitude,
                    longitude: start.longitude,
                },
                Coordinates {
                    latitude: end.latitude,
                    longitude: end.longitude,
                },
            )
        }
    }

    pub fn track(&self) -> impl Iterator<Item = Coordinates> + '_ {
        self.points.iter().map(|p| Coordinates {
            latitude: p.latitude,
            longitude: p.longitude,
        })
    }
}

pub fn load(buf: &[u8]) -> Result<SMLAttempt, serde_json::Error> {
    Ok(serde_json::from_reader::<_, SMLDoc>(buf)?.attempt)
}
