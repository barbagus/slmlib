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

//! A to interact with straight line missions related file formats
#![no_std]

#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "fix")]
pub mod fix;
#[cfg(feature = "gpx")]
pub mod gpx;
#[cfg(feature = "sml")]
pub mod sml;
