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

use super::geo::{vincenty_inverse, Geodesic, Point, Sequence};

#[derive(Debug, Clone)]
pub struct PointStats {
    pub deviation: f64,
    pub made_good: f64,
}

pub struct TrackStats {
    pub route_length: f64,
    pub max_deviation: f64,
    pub points: Vec<PointStats>,
}

/// Vincenty's inverse formula with required precision
pub fn geodetic_distance(p1: Point, p2: Point) -> Option<f64> {
    vincenty_inverse(p1, p2, 100, 1e-9)
}

/// Compute statistics for the line and every given points
pub fn compute_stats<I>(route: (Point, Point), track: I) -> TrackStats
where
    I: IntoIterator<Item = Point>,
{
    let (start, end) = route;
    let route = Geodesic::new(start, end);
    let route_length = geodetic_distance(start, end).unwrap();
    let mut max_deviation = 0_f64;

    let points = track
        .into_iter()
        .map(|point| {
            let (projection, sequence, _side) = point.project_onto(route);

            match sequence {
                Sequence::BeforeStart => PointStats {
                    deviation: 0.0,
                    made_good: 0.0,
                },
                Sequence::AfterEnd => PointStats {
                    deviation: 0.0,
                    made_good: route_length,
                },
                Sequence::InBetween => {
                    let deviation = geodetic_distance(projection, point).unwrap();
                    if deviation > max_deviation {
                        max_deviation = deviation;
                    }
                    PointStats {
                        deviation,
                        made_good: geodetic_distance(start, projection).unwrap(),
                    }
                }
            }
        })
        .collect();

    TrackStats {
        route_length,
        max_deviation,
        points,
    }
}
