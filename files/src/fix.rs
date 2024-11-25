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

use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

/// "meta" file: the burdell "level" (not sure how to qualify this)
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum BurdellLevel {
    Pro,
    Amateur,
    Newbie,
}
impl BurdellLevel {
    pub fn each() -> impl Iterator<Item = Self> {
        [
            BurdellLevel::Pro,
            BurdellLevel::Amateur,
            BurdellLevel::Newbie,
        ]
        .into_iter()
    }
}

/// "meta" file: the leniency level (percentage of worst points ignored)
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LeniencyLevel {
    /// Percentage of points dropped (if any)
    pub ignore: Option<i32>,
    /// Maximum deviation from route in meters.
    pub max_deviation: f64,
    /// Medal ranking (if any)
    pub medal: Option<String>,
    /// A `BurdellLevel` to burdell score mapping
    pub scores: HashMap<BurdellLevel, f64>,
}

/// "meta" file: the document "root" object
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SMLScores {
    /// The route length in kilometers
    pub route_length: f64,
    /// Per leniency-level scores
    pub scores: Vec<LeniencyLevel>,
}

pub fn load<P>(path: P) -> SMLScores
where
    P: AsRef<Path>,
{
    let rdr = File::open(path).unwrap();
    let rdr = BufReader::new(rdr);
    serde_json::from_reader::<_, SMLScores>(rdr).unwrap()
}
