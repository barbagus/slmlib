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

/// A geographical coordinates pair.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// Latitude in radians [-PI/2, PI/2] (negative means South).
    pub(crate) lat_rad: f64,
    /// Longitude in radians [-PI, PI] (negative means West).
    pub(crate) lon_rad: f64,
}

impl Point {
    /// Build from geographical coordinates expressed in degrees.
    ///
    /// Latitudes go from -90° to 90° (both included) with negative values for the southern
    /// hemisphere.
    ///
    /// Longitudes go from -180° (excluded) to 180° (included) with negative values for the western
    /// hemisphere.
    ///
    /// Returns `None` if any of the coordinates are out of range.
    pub fn new(latitude: f64, longitude: f64) -> Option<Self> {
        if -90_f64 <= latitude && latitude <= 90_f64 && -180_f64 < longitude && longitude <= 180_f64
        {
            Some(Self {
                lat_rad: latitude.to_radians(),
                lon_rad: longitude.to_radians(),
            })
        } else {
            None
        }
    }

    /// Approximate equality down `f64::EPSILON`
    pub fn quasi_eq(&self, other: Self) -> bool {
        self.approx_eq(other, f64::EPSILON)
    }

    /// Approximate equality down `epsilon`
    pub fn approx_eq(&self, other: Self, epsilon: f64) -> bool {
        (self.lat_rad - other.lat_rad).abs() < epsilon
            && (self.lon_rad - other.lon_rad).abs() < epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! spot_on_tests {
        ($($name:ident: ($lat:expr,$lon:expr),)*) => {
        $(
            #[test]
            fn $name() {
                assert!(Point::new($lat, $lon).is_some())
            }
        )*
        }
    }

    spot_on_tests! {
        spot_on_lat_high: (90.0, 21.83),
        spot_on_lat_low: (-90.0, 21.83),
        spot_on_lon_high: (56.15, 180.0),
    }

    macro_rules! out_of_range_tests {
        ($($name:ident: ($lat:expr,$lon:expr),)*) => {
        $(
            #[test]
            fn $name() {
                assert!(Point::new($lat, $lon).is_none())
            }
        )*
        }
    }

    out_of_range_tests! {
        out_of_range_lat_over: (107.3, 21.83),
        out_of_range_lat_under: (-118.2, 21.83),
        out_of_range_lon_over: (56.15, 190.18),
        out_of_range_lon_under: (47.21, -180.0),
    }
}
