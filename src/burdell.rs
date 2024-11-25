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

use crate::stats::TrackStats;
use alloc::{vec, vec::Vec};
use libm::{ceil, floor, log10, pow};

///
/// A Burdell score penalty setting.
///
#[derive(Debug, Clone, Copy)]
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
pub fn compute_burdell_score(config: BurdellSettings, stats: &TrackStats) -> f64 {
    let steps = ceil(stats.route_length / config.step) as usize;

    let mut slots: Vec<Option<f64>> = vec![None; steps];

    let mut used_slots: Vec<usize> = Vec::with_capacity(slots.len());

    for p in stats.points.iter() {
        let i = floor(p.made_good / config.step) as usize;
        let slot = slots.get_mut(i).unwrap();

        match slot {
            Some(max_deviation) => {
                if p.deviation > *max_deviation {
                    slot.replace(p.deviation);
                }
            }
            None => {
                used_slots.push(i);
                slot.replace(p.deviation);
            }
        };
    }

    used_slots.sort_unstable();

    for (i1, i2) in core::iter::zip(
        used_slots.iter().cloned(),
        used_slots.iter().skip(1).cloned(),
    ) {
        if i2 - i1 > 1 {
            let fill = (slots[i1].unwrap() + slots[i2].unwrap()) / 2.0;
            for slot in slots.iter_mut().take(i2).skip(i1 + 1) {
                slot.replace(fill);
            }
        }
    }

    for slot in slots.iter_mut().take(*used_slots.first().unwrap()) {
        slot.replace(0_f64);
    }
    for slot in slots.iter_mut().skip(*used_slots.last().unwrap() + 1) {
        slot.replace(0_f64);
    }

    let log = log10(stats.route_length);
    let mut penalities: f64 = 0.0;
    for slot in slots {
        penalities += 100.0 * pow(slot.unwrap() / config.coefficient, log);
    }

    f64::max(100.0 - penalities, 0.0)
}
