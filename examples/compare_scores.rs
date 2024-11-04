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
// You should have received a copy of the GNU General Public License along with Foobar. If not, see
// <https://www.gnu.org/licenses/>.

use serde::Deserialize;
use serde_json;
use slmlib::{self, files::sml::SMLDoc};
use std::{collections::HashMap, fmt::Debug, fs::read_dir};

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
enum Medal {
    Platinum,
    Gold,
    Silver,
    Bronze,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
enum Level {
    Pro,
    Amateur,
    Newbie,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Score {
    ignore: Option<i32>,
    max_deviation: f64,
    medal: Option<Medal>,
    levels: HashMap<Level, f64>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Meta {
    target_line_distance: f64,
    scores: Vec<Score>,
}

fn main() {
    println!(
        "{}",
        [
            "mission",
            "level",
            "target_line_length[orig]",
            "target_line_length[barb]",
            "target_line_length[diff]",
            "max_deviation[orig]",
            "max_deviation[barb]",
            "max_deviation[diff]",
            "burdell_score[orig]",
            "burdell_score[barb]",
            "burdell_score[diff]",
        ]
        .join(";")
    );

    for sml_path in read_dir("fixtures")
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sml"))
    // .filter(|p| p.file_stem().unwrap() == "geowizard-wales1a")
    {
        let name = sml_path.file_stem().unwrap();
        let scores_path = sml_path.clone().with_extension("json");

        let sml =
            serde_json::from_reader::<_, SMLDoc>(std::fs::File::open(sml_path.clone()).unwrap())
                .unwrap()
                .attempt;

        let target_line = sml.target_line();
        let points = sml.geo_points().collect::<Vec<_>>();

        let meta =
            serde_json::from_reader::<_, Meta>(std::fs::File::open(scores_path).unwrap()).unwrap();

        for score in meta.scores.into_iter().filter(|s| s.ignore.is_none()) {
            for level in [Level::Pro, Level::Amateur, Level::Newbie].into_iter() {
                let burdell_score = score.levels[&level];
                let conf = slmlib::Config {
                    burdell: match level {
                        Level::Pro => slmlib::LVL_PRO,
                        Level::Amateur => slmlib::LVL_AMATEUR,
                        Level::Newbie => slmlib::LVL_NEWBIE,
                    },
                };
                let res = slmlib::score_my_line(conf, target_line, points.iter().cloned());

                println!(
                    "{}",
                    [
                        format!("{:?}", name),
                        format!("{:?}", level),
                        //
                        format!("{:.2}", meta.target_line_distance),
                        format!("{:.2}", res.target_line_length / 1000_f64),
                        format!(
                            "{:.2}",
                            (meta.target_line_distance - res.target_line_length / 1000_f64).abs()
                        ),
                        //
                        format!("{:.1}", score.max_deviation),
                        format!("{:.1}", res.max_deviation),
                        format!("{:.1}", (score.max_deviation - res.max_deviation).abs()),
                        //
                        format!("{:.2}", burdell_score),
                        format!("{:.2}", res.burdell),
                        format!("{:.2}", (burdell_score - res.burdell).abs()),
                    ]
                    .join(";")
                );
            }
        }
    }
}
