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

mod burdell;
mod geo;
mod stats;

pub use burdell::{compute_burdell_score, LVL_AMATEUR, LVL_NEWBIE, LVL_PRO};
pub use geo::Point;
pub use stats::{compute_stats, PointStats, TrackStats};

/// The medal color associated with a max deviation value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MedalRank {
    /// Max deviation less than 25 meters.
    Platinum,
    /// Max deviation less than 50 meters.
    Gold,
    /// Max deviation less than 75 meters.
    Silver,
    /// Max deviation less than 100 meters.
    Bronze,
}
pub fn compute_medal_rank(stats: &TrackStats) -> Option<MedalRank> {
    let value = stats.max_deviation;
    if value < 25.0 {
        Some(MedalRank::Platinum)
    } else if value < 50.0 {
        Some(MedalRank::Gold)
    } else if value < 75.0 {
        Some(MedalRank::Silver)
    } else if value < 100.0 {
        Some(MedalRank::Bronze)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use std::path::PathBuf;

    macro_rules! mission_tests {
        ($($name:ident: $path:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let sml_path = PathBuf::from($path);
                let sml = files::sml::load(&sml_path);
                let route = {
                    let (start, end) = sml.route();
                    let start = Point::new(start.0, start.1);
                    let end = Point::new(end.0, end.1);
                    (start, end)
                };

                let stats = compute_stats(route, sml.track().map(|t| Point::new(t.0, t.1)));
                let scores = files::fix::load(sml_path.with_extension("json"));

                assert_abs_diff_eq!(stats.route_length, sml.target_line_length, epsilon = 1e-2);
                assert_abs_diff_eq!(
                    stats.route_length / 1000_f64,
                    scores.route_length,
                    epsilon = 1e-2
                );

                assert_eq!(
                    match compute_medal_rank(&stats) {
                        Some(MedalRank::Platinum) => Some(String::from("PLATINUM")),
                        Some(MedalRank::Gold) => Some(String::from("GOLD")),
                        Some(MedalRank::Silver) => Some(String::from("SILVER")),
                        Some(MedalRank::Bronze) => Some(String::from("BRONZE")),
                        None => None,
                    },
                    scores.scores[0].medal
                );

                let mut max_deviation = 0_f64;

                for (point_stats, sml_point) in std::iter::zip(stats.points.iter(), sml.points.iter()) {
                    assert_abs_diff_eq!(
                        point_stats.deviation,
                        sml_point.distance_to_line,
                        epsilon = 1e-2,
                    );
                    assert_abs_diff_eq!(
                        point_stats.made_good,
                        sml_point.control_point_distance_to_start,
                        epsilon = 1e-2
                    );

                    if sml_point.distance_to_line > max_deviation {
                        max_deviation = sml_point.distance_to_line;
                    }
                }

                assert_abs_diff_eq!(stats.max_deviation, max_deviation, epsilon = 1e-2);
                assert_abs_diff_eq!(
                    stats.max_deviation,
                    scores.scores[0].max_deviation,
                    epsilon = 1e-1
                );
            }
        )*
        }
    }
    mission_tests! {
        mission_archie_iom: "fixtures/archie-iom.sml",
        mission_archie_scotland: "fixtures/archie-scotland.sml",
        mission_archie_wales_run: "fixtures/archie-wales-run.sml",
        mission_archie_wales_walk: "fixtures/archie-wales-walk.sml",
        mission_geowizard_iom: "fixtures/geowizard-iom.sml",
        mission_geowizard_norway: "fixtures/geowizard-norway.sml",
        mission_geowizard_scotland: "fixtures/geowizard-scotland.sml",
        mission_geowizard_wales1a: "fixtures/geowizard-wales1a.sml",
        mission_geowizard_wales1b: "fixtures/geowizard-wales1b.sml",
        mission_geowizard_wales2: "fixtures/geowizard-wales2.sml",
        mission_geowizard_wales3: "fixtures/geowizard-wales3.sml",
        mission_geowizard_wales4: "fixtures/geowizard-wales4.sml",
        mission_hiiumaa: "fixtures/hiiumaa.sml",
        mission_muhu: "fixtures/muhu.sml",
        mission_new_forest: "fixtures/new-forest.sml",
        mission_saaremaa: "fixtures/saaremaa.sml",
        mission_schaffhausen: "fixtures/schaffhausen.sml",
    }
}
