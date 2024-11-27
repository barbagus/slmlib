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

/// The medal color associated with a max deviation value.
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
pub fn compute_rank(stats: &Slm) -> Option<Rank> {
    let value = stats.max_deviation;
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
