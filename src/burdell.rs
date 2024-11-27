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

extern crate alloc;

use crate::{Deviation, Point, Progress, Slm};
use alloc::{vec, vec::Vec};
use core::iter;
use libm::{floor, log10, pow};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

///
/// A Burdell score penalty setting.
///
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BurdellSettings {
    /// Distance in meters over which each penalty term is computed (small is more severe).
    step: f64,
    /// A magic number (small is more severe).
    coefficient: f64,
}

/// The "Pro" penalty settings.
pub const LVL_PRO: BurdellSettings = BurdellSettings {
    step: 1.0,
    coefficient: 150.0,
};

/// The "Amateur" penalty settings.
pub const LVL_AMATEUR: BurdellSettings = BurdellSettings {
    step: 5.0,
    coefficient: 175.0,
};

/// The "Newbie" penalty settings.
pub const LVL_NEWBIE: BurdellSettings = BurdellSettings {
    step: 25.0,
    coefficient: 200.0,
};

///
/// Burdell score computation
///
pub fn compute_score(config: BurdellSettings, slm: &Slm) -> f64 {
    let segment_count = floor(slm.route_length / config.step) + 1.0;

    let mut segments: Vec<Option<f64>> = vec![None; segment_count as usize];
    let mut filled_segments: Vec<usize> = Vec::with_capacity(segments.len());

    segments.first_mut().unwrap().replace(0.0);
    filled_segments.push(0);

    segments.last_mut().unwrap().replace(0.0);
    filled_segments.push(segments.len() - 1);

    for point in slm.track.iter() {
        let Point { progress, .. } = point;
        let (made_good, deviation) = match progress {
            Progress::Standby => continue,
            Progress::EnRoute {
                deviation,
                made_good,
                ..
            } => match deviation {
                Some(deviation) => match deviation {
                    Deviation::Left(deviation) => (*made_good, *deviation),
                    Deviation::Right(deviation) => (*made_good, *deviation),
                },
                None => (*made_good, 0.0),
            },
            Progress::Arrived => continue,
        };

        // Trivial segment division: get the best overall results
        // (we tried "centering" the segments on the total route, it gets worse)
        let segment_index = floor(made_good / config.step) as usize;
        let segment = segments.get_mut(segment_index).unwrap();

        match segment {
            Some(max_deviation) => {
                if deviation > *max_deviation {
                    segment.replace(deviation);
                }
            }
            None => {
                filled_segments.push(segment_index);
                segment.replace(deviation);
            }
        };
    }

    filled_segments.sort_unstable();

    for (i1, i2) in iter::zip(
        filled_segments.iter().cloned(),
        filled_segments.iter().skip(1).cloned(),
    ) {
        if i2 - i1 > 1 {
            let fill = (segments[i1].unwrap() + segments[i2].unwrap()) / 2.0;
            for segment in segments.iter_mut().take(i2).skip(i1 + 1) {
                segment.replace(fill);
            }
        }
    }

    let log = log10(slm.route_length);
    let mut penalities: f64 = 0.0;
    for s in segments {
        penalities += 100.0 * pow(s.unwrap() / config.coefficient, log);
    }

    f64::max(100.0 - penalities, 0.0)
}
