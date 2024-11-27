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
#![no_std]
extern crate alloc;

pub mod burdell;
pub mod files;
mod geo;
pub mod geowizard;
mod slm;

pub use slm::*;

macro_rules! vincenty_inverse {
    ($p1: expr, $p2: expr) => {
        geo::vincenty_inverse($p1, $p2, 100, 1e-9)
    };
}

impl From<Coordinates> for geo::Point {
    fn from(value: Coordinates) -> Self {
        geo::Point::new(value.latitude, value.longitude)
    }
}

impl From<geo::Point> for Coordinates {
    fn from(value: geo::Point) -> Self {
        let value = value.coordinates();
        Coordinates {
            latitude: value.0,
            longitude: value.1,
        }
    }
}

/// Analyze a straight line mission
pub fn analyze<I>(start: Coordinates, end: Coordinates, track: I) -> Slm
where
    I: IntoIterator<Item = Coordinates>,
{
    let g_start: geo::Point = start.into();
    let g_end: geo::Point = end.into();

    let g_route = geo::Geodesic::new(g_start, g_end);
    let route_length = vincenty_inverse!(g_start, g_end).unwrap();

    let mut max_deviation = 0_f64;

    let track = track
        .into_iter()
        .map(|coordinates| {
            let g_point: geo::Point = coordinates.clone().into();
            let (g_projection, order, side) = g_point.project_onto(g_route);

            Point {
                coordinates,
                progress: match order {
                    geo::Order::Before => Progress::Standby,
                    geo::Order::Between => {
                        let deviation = vincenty_inverse!(g_projection, g_point).unwrap();
                        if deviation > max_deviation {
                            max_deviation = deviation;
                        }
                        Progress::EnRoute {
                            on_route: g_projection.into(),
                            made_good: vincenty_inverse!(g_start, g_projection).unwrap(),
                            deviation: match side {
                                geo::Side::Left => Some(Deviation::Left(deviation)),
                                geo::Side::Right => Some(Deviation::Right(deviation)),
                                geo::Side::Center => None,
                            },
                        }
                    }
                    geo::Order::After => Progress::Arrived,
                },
            }
        })
        .collect();

    Slm {
        route_start: start,
        route_end: end,
        route_length,
        max_deviation,
        track,
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use approx::assert_abs_diff_eq;
    use core::iter;
    use std::{fs, path};

    fn mission_test(name: &str) {
        let base = path::Path::new("fixtures");

        let them = {
            let path = base.join(name).with_extension("sml");
            let buf = fs::read(path).expect("read SML file");
            files::sml::load(&buf).expect("parse SML file")
        };
        let (start, end) = them.route();

        let us = analyze(start, end, them.track());

        assert_abs_diff_eq!(us.route_length, them.target_line_length, epsilon = 1e-2);

        let mut them_max_deviation = 0_f64;

        for (us_point, them_point) in iter::zip(us.track.into_iter(), them.points.into_iter()) {
            let Point { progress, .. } = us_point;

            let (us_projection, us_made_good, us_deviation) = match progress {
                Progress::Standby => (start, 0.0, 0.0),
                Progress::EnRoute {
                    deviation,
                    on_route,
                    made_good,
                } => match deviation {
                    Some(deviation) => match deviation {
                        Deviation::Left(deviation) => (on_route, made_good, deviation),
                        Deviation::Right(deviation) => (on_route, made_good, deviation),
                    },
                    None => (on_route, made_good, 0.0),
                },
                Progress::Arrived => (end, us.route_length, 0.0),
            };

            assert_abs_diff_eq!(
                us_projection.latitude,
                them_point.control_point_latitude,
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                us_projection.longitude,
                them_point.control_point_longitude,
                epsilon = 1e-6
            );

            assert_abs_diff_eq!(
                us_made_good,
                them_point.control_point_distance_to_start,
                epsilon = 1e-2
            );

            assert_abs_diff_eq!(us_deviation, them_point.distance_to_line, epsilon = 1e-2,);

            if them_point.distance_to_line > them_max_deviation {
                them_max_deviation = them_point.distance_to_line;
            }
        }

        assert_abs_diff_eq!(us.max_deviation, them_max_deviation, epsilon = 1e-2);
    }

    macro_rules! mission_tests {
        ($($f:ident: $n:expr,)*) => {
        $(
            #[test]
            fn $f() {
                mission_test($n)
            }
        )*
        }
    }
    mission_tests! {
        mission_archie_iom: "archie-iom",
        mission_archie_scotland: "archie-scotland",
        mission_archie_wales_run: "archie-wales-run",
        mission_archie_wales_walk: "archie-wales-walk",
        mission_geowizard_iom: "geowizard-iom",
        mission_geowizard_norway: "geowizard-norway",
        mission_geowizard_scotland: "geowizard-scotland",
        mission_geowizard_wales1a: "geowizard-wales1a",
        mission_geowizard_wales1b: "geowizard-wales1b",
        mission_geowizard_wales2: "geowizard-wales2",
        mission_geowizard_wales3: "geowizard-wales3",
        mission_geowizard_wales4: "geowizard-wales4",
        mission_hiiumaa: "hiiumaa",
        mission_muhu: "muhu",
        mission_new_forest: "new-forest",
        mission_saaremaa: "saaremaa",
        mission_schaffhausen: "schaffhausen",
    }
}
