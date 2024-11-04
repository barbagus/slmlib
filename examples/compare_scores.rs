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

//! A tool to compute and display the relative correctness of our implementation with regards to the
//! original implementation over at [scoremyline.com](https://scoremyline.com)
//!
//! The expected results have been manually collected from the site and organized in so called
//! "meta" files.

use approx::assert_relative_eq;
use core::f64;
use serde::Deserialize;
use serde_json;
use slmlib::{self, files::sml::SMLDoc, BurdellSettings};
use std::{collections::HashMap, fmt::Debug, fs::read_dir};

/// "meta" file: the burdell "level" (not sure how to qualify this)
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum BurdellLevel {
    Pro,
    Amateur,
    Newbie,
}

/// "meta" file: the leniency level (percentage of worst points ignored)
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct LeniencyLevel {
    /// the percentage of points dropped (if any)
    ignore: Option<i32>,
    /// Maximum deviation from target line in meters.
    max_deviation: f64,
    /// A `BurdellLevel` to burdell score mapping
    scores: HashMap<BurdellLevel, f64>,
}

/// "meta" file: the document "root" object
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct MetaDoc {
    /// The target line length in kilometers
    target_line_length: f64,
    /// The target line length in kilometers
    scores: Vec<LeniencyLevel>,
}

struct Record {
    mission: String,
    distance: f64,
    distance_diff: f64,
    max_dev: f64,
    max_dev_diff: f64,
    burdell: HashMap<BurdellLevel, (f64, f64)>,
}

impl From<BurdellLevel> for BurdellSettings {
    fn from(value: BurdellLevel) -> Self {
        match value {
            BurdellLevel::Pro => slmlib::LVL_PRO,
            BurdellLevel::Amateur => slmlib::LVL_AMATEUR,
            BurdellLevel::Newbie => slmlib::LVL_NEWBIE,
        }
    }
}

impl BurdellLevel {
    fn iterate() -> impl Iterator<Item = Self> {
        [
            BurdellLevel::Pro,
            BurdellLevel::Amateur,
            BurdellLevel::Newbie,
        ]
        .into_iter()
    }
}

fn main() {
    let paths = read_dir("fixtures")
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sml"))
        .collect::<Vec<_>>();

    let mut records: Vec<Record> = Vec::with_capacity(paths.len());

    for (i, sml_path) in paths.iter().enumerate() {
        let name = sml_path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        eprintln!("({}/{}) {}", i + 1, paths.len(), name);

        let scores_path = sml_path.clone().with_extension("json");

        let (target_line, points) = {
            let rdr = std::fs::File::open(sml_path.clone()).unwrap();
            let attempt = serde_json::from_reader::<_, SMLDoc>(rdr).unwrap().attempt;

            (
                attempt.target_line(),
                attempt.geo_points().collect::<Vec<_>>(),
            )
        };

        let meta_doc = {
            let rdr = std::fs::File::open(scores_path).unwrap();
            serde_json::from_reader::<_, MetaDoc>(rdr).unwrap()
        };

        for score in meta_doc.scores.into_iter().filter(|s| s.ignore.is_none()) {
            let results = BurdellLevel::iterate()
                .map(|level| {
                    let conf = slmlib::Config {
                        burdell: level.into(),
                    };
                    (
                        level,
                        (slmlib::score_my_line(conf, target_line, points.iter().cloned())),
                    )
                })
                .collect::<HashMap<_, _>>();

            assert_relative_eq!(
                results[&BurdellLevel::Pro].target_line_length,
                results[&BurdellLevel::Amateur].target_line_length,
                epsilon = f64::EPSILON
            );
            assert_relative_eq!(
                results[&BurdellLevel::Pro].target_line_length,
                results[&BurdellLevel::Newbie].target_line_length,
                epsilon = f64::EPSILON
            );

            assert_relative_eq!(
                results[&BurdellLevel::Pro].max_deviation,
                results[&BurdellLevel::Amateur].max_deviation,
                epsilon = f64::EPSILON
            );
            assert_relative_eq!(
                results[&BurdellLevel::Pro].max_deviation,
                results[&BurdellLevel::Newbie].max_deviation,
                epsilon = f64::EPSILON
            );

            records.push(Record {
                mission: name.clone(),
                distance: results[&BurdellLevel::Pro].target_line_length / 1000_f64,
                distance_diff: (results[&BurdellLevel::Pro].target_line_length / 1000_f64
                    - meta_doc.target_line_length),
                max_dev: results[&BurdellLevel::Pro].max_deviation,
                max_dev_diff: results[&BurdellLevel::Pro].max_deviation - score.max_deviation,
                burdell: results
                    .iter()
                    .map(|(level, result)| {
                        (
                            level.clone(),
                            (result.burdell, result.burdell - score.scores[level]),
                        )
                    })
                    .collect(),
            });
        }
    }
    println!("## Distances");
    println!(
        "| {} |",
        [
            "Mission           ",
            "Distance (km) ",
            "Distance err. ",
            "Max. dev. (m) ",
            "Max. dev. err.",
        ]
        .join(" | ")
    );
    println!(
        "|{}|",
        [
            ":-------------------",
            "---------------:",
            "---------------:",
            "---------------:",
            "---------------:",
        ]
        .join("|")
    );
    for record in records.iter() {
        println!(
            "| {} |",
            [
                format!("{:18}", record.mission),
                format!("{:14.2}", record.distance),
                format!("{:14.2}", record.distance_diff),
                format!("{:14.1}", record.max_dev),
                format!("{:14.1}", record.max_dev_diff),
            ]
            .join(" | ")
        );
    }
    println!("");
    println!("## Burdell scores");
    println!(
        "| {} |",
        [
            "Mission           ",
            "Pro         ",
            "Pro err.    ",
            "Amateur     ",
            "Amateur err.",
            "Newbie      ",
            "Newbie err. ",
        ]
        .join(" | ")
    );
    println!(
        "|{}|",
        [
            ":-------------------",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
        ]
        .join("|")
    );
    for record in records.iter() {
        println!(
            "| {} |",
            [
                format!("{:18}", record.mission),
                format!("{:12.2}", record.burdell[&BurdellLevel::Pro].0),
                format!("{:12.2}", record.burdell[&BurdellLevel::Pro].1),
                format!("{:12.2}", record.burdell[&BurdellLevel::Amateur].0),
                format!("{:12.2}", record.burdell[&BurdellLevel::Amateur].1),
                format!("{:12.2}", record.burdell[&BurdellLevel::Newbie].0),
                format!("{:12.2}", record.burdell[&BurdellLevel::Newbie].1),
            ]
            .join(" | ")
        );
    }
}
