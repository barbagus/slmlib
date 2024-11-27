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

use super::Slm;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The rank associated with a max deviation value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Rank {
    /// Max deviation less than 25 meters.
    Platinum,
    /// Max deviation less than 50 meters.
    Gold,
    /// Max deviation less than 75 meters.
    Silver,
    /// Max deviation less than 100 meters.
    Bronze,
}

impl Rank {
    fn from_deviation(value: f64) -> Option<Self> {
        if value < 25.0 {
            Some(Rank::Platinum)
        } else if value < 50.0 {
            Some(Rank::Gold)
        } else if value < 75.0 {
            Some(Rank::Silver)
        } else if value < 100.0 {
            Some(Rank::Bronze)
        } else {
            None
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Rank::Platinum => "PLATINUM",
            Rank::Gold => "GOLD",
            Rank::Silver => "SILVER",
            Rank::Bronze => "BRONZE",
        }
    }
}

pub fn compute_rank(slm: &Slm) -> Option<Rank> {
    Rank::from_deviation(slm.max_deviation)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::super::*;
    use super::*;
    use std::{fs, path};

    fn rank_test(name: &str) {
        let base = path::Path::new("fixtures");

        let sml = {
            let sml_path = base.join(name).with_extension("sml");
            let buf = fs::read(sml_path).expect("read SML file");
            files::sml::load(&buf).expect("parse SML file")
        };

        let fix = {
            let path = base.join(name).with_extension("json");
            let buf = fs::read(path).expect("read FIX file");
            files::fix::load(&buf).expect("parse FIX file")
        };

        let (start, end) = sml.route();
        let mission = analyze(start, end, sml.track());

        for score in fix.scores {
            if score.ignore.is_none() {
                let rank: Option<&str> = compute_rank(&mission).map(|r| r.to_str());
                assert_eq!(score.medal.as_ref().map(|s| s.as_str()), rank);
            }
        }
    }

    macro_rules! rank_tests {
        ($($f:ident: $n:expr,)*) => {
        $(
            #[test]
            fn $f() {
                rank_test($n)
            }
        )*
        }
    }
    rank_tests! {
        rank_archie_iom: "archie-iom",
        rank_archie_scotland: "archie-scotland",
        rank_archie_wales_run: "archie-wales-run",
        rank_archie_wales_walk: "archie-wales-walk",
        rank_geowizard_iom: "geowizard-iom",
        rank_geowizard_norway: "geowizard-norway",
        rank_geowizard_scotland: "geowizard-scotland",
        rank_geowizard_wales1a: "geowizard-wales1a",
        rank_geowizard_wales1b: "geowizard-wales1b",
        rank_geowizard_wales2: "geowizard-wales2",
        rank_geowizard_wales3: "geowizard-wales3",
        rank_geowizard_wales4: "geowizard-wales4",
        rank_hiiumaa: "hiiumaa",
        rank_muhu: "muhu",
        rank_new_forest: "new-forest",
        rank_saaremaa: "saaremaa",
        rank_schaffhausen: "schaffhausen",
    }
}
