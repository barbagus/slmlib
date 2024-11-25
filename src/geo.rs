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

//! Geographic utilities library
use libm::{asin, atan, atan2, fabs, sincos, sqrt, tan};

///
/// A geographical point.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Latitude in radians (negative means South).
    lat_rad: f64,
    /// Longitude in radians (negative means West).
    lon_rad: f64,
}

impl Point {
    /// Build from geographical coordinates in decimal degrees; north and east as positive values,
    /// south and west as negative values.
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            lat_rad: latitude.to_radians(),
            lon_rad: longitude.to_radians(),
        }
    }

    /// Approximate equality down `f64::EPSILON`
    fn quasi_eq(&self, other: Self) -> bool {
        self.approx_eq(other, f64::EPSILON)
    }

    /// Approximate equality down `epsilon`
    fn approx_eq(&self, other: Self, epsilon: f64) -> bool {
        fabs(self.lat_rad - other.lat_rad) < epsilon && fabs(self.lon_rad - other.lon_rad) < epsilon
    }
}

///
/// A 3D cartesian representation of a point on a WSG84 ellipsoid
///
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    /// Cross product
    fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Dot product
    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Subtraction
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Scaling
    fn mul(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    /// Length
    fn len(self) -> f64 {
        sqrt(self.dot(self))
    }

    /// convert to unit vector with same direction
    fn to_unit(self) -> Self {
        self.mul(self.len().recip())
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        let (sin_lat, cos_lat) = sincos(value.lat_rad);
        let (sin_lon, cos_lon) = sincos(value.lon_rad);

        let r: f64 = AB / sqrt(B2 * cos_lat * cos_lat + A2 * sin_lat * sin_lat);

        Vector {
            x: cos_lon * cos_lat * r,
            y: sin_lon * cos_lat * r,
            z: sin_lat * r,
        }
    }
}

impl From<Vector> for Point {
    fn from(value: Vector) -> Self {
        let value = value.to_unit();

        Point {
            lat_rad: asin(value.z),
            lon_rad: atan2(value.y, value.x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geodesic {
    v_start: Vector,
    v_end: Vector,
    v_normal: Vector,
}

impl Geodesic {
    pub fn new(start: Point, end: Point) -> Self {
        let v_start: Vector = start.into();
        let v_end: Vector = end.into();
        Self {
            v_start,
            v_end,
            v_normal: v_end.cross(v_start).to_unit(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sequence {
    BeforeStart,
    AfterEnd,
    InBetween,
}

impl Point {
    /// Project the point orthogonally to a geodesic
    ///
    /// Return a tuple comprising of:
    ///   - the projected point on the geodesic
    ///   - the sequence of the projected point with regards to `start` and `end` of the geodesic
    ///   - the side of the original point (from `start`, looking towards `end` of the geodesic)
    pub fn project_onto(self, geodesic: Geodesic) -> (Self, Sequence, Side) {
        let v_point: Vector = self.into();

        let f = v_point.dot(geodesic.v_normal);

        let v_projection = v_point.sub(geodesic.v_normal.mul(f));

        let h1 = v_projection.cross(geodesic.v_start).dot(geodesic.v_normal);
        let h2 = v_projection.cross(geodesic.v_end).dot(geodesic.v_normal);

        (
            v_projection.into(),
            if h1 < 0.0 {
                Sequence::BeforeStart
            } else if h2 > 0.0 {
                Sequence::AfterEnd
            } else {
                Sequence::InBetween
            },
            if f > 0.0 {
                Side::Right
            } else if f < 0.0 {
                Side::Left
            } else {
                Side::Center
            },
        )
    }
}

const A: f64 = 6378137.0;
const F: f64 = 1.0 / 298.257223563;
const B: f64 = A * (1.0 - F);

const A2: f64 = A * A;
const B2: f64 = B * B;
const AB: f64 = A * B;

///
/// Compute the geodetic distance between two points, using
/// [Vincenty's formulae](https://en.wikipedia.org/wiki/Vincenty's_formulae).
///
pub fn vincenty_inverse(p1: Point, p2: Point, max_iteration: i32, accuracy: f64) -> Option<f64> {
    let big_l = p2.lon_rad - p1.lon_rad;

    let (sin_u1, cos_u1) = sincos(atan((1_f64 - F) * tan(p1.lat_rad)));
    let (sin_u2, cos_u2) = sincos(atan((1_f64 - F) * tan(p2.lat_rad)));

    let mut lambda = big_l;

    for _ in 0..max_iteration {
        let (sin_lambda, cos_lambda) = sincos(lambda);

        let sin_sigma = sqrt(
            (cos_u2 * sin_lambda) * (cos_u2 * sin_lambda)
                + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda)
                    * (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda),
        );

        if sin_sigma == 0_f64 {
            return if p1.quasi_eq(p2) {
                // coincident points
                Some(0_f64)
            } else {
                // antipodal points
                None
            };
        }

        let cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;

        let sigma = atan2(sin_sigma, cos_sigma);

        let sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;

        let cos2_alpha = 1_f64 - sin_alpha * sin_alpha;

        let cos_2sigma_m = if cos2_alpha == 0_f64 {
            0_f64
        } else {
            cos_sigma - 2_f64 * sin_u1 * sin_u2 / cos2_alpha
        };

        let c = F / 16_f64 * cos2_alpha * (4_f64 + F * (4_f64 - 3_f64 * cos2_alpha));

        let lambda_prev = lambda;

        lambda = big_l
            + (1_f64 - c)
                * F
                * sin_alpha
                * (sigma
                    + c * sin_sigma
                        * (cos_2sigma_m
                            + c * cos_sigma * (-1_f64 + 2_f64 * cos_2sigma_m * cos_2sigma_m)));

        if fabs(lambda - lambda_prev) <= accuracy {
            let u2 = cos2_alpha * (A2 - B2) / (B2);
            let a = 1_f64
                + u2 / 16384_f64 * (4096_f64 + u2 * (-768_f64 + u2 * (320_f64 - 175_f64 * u2)));
            let b = u2 / 1024_f64 * (256_f64 + u2 * (-128_f64 + u2 * (74_f64 - 47_f64 * u2)));

            let delta_sigma = b
                * sin_sigma
                * (cos_2sigma_m
                    + b / 4_f64
                        * (cos_sigma * (-1_f64 + 2_f64 * cos_2sigma_m * cos_2sigma_m)
                            - b / 6_f64
                                * cos_2sigma_m
                                * (-3_f64 + 4_f64 * sin_sigma * sin_sigma)
                                * (-3_f64 + 4_f64 * cos_2sigma_m * cos_2sigma_m)));

            let s = B * a * (sigma - delta_sigma);
            return Some(s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    macro_rules! point_conversion_tests {
        ($($name:ident: ($lat:expr,$lon:expr))*) => {
        $(
            #[test]
            fn $name() {
                let p = Point::new($lat, $lon);
                let v: Vector = p.into();
                let q: Point = v.into();
                assert!(p.approx_eq(q, 1e-12))
            }
        )*
        }
    }
    point_conversion_tests! {
        point_conversion_ne: (78.29, 21.83)
        point_conversion_nee: (56.15, 145.18)
        point_conversion_nw: (47.21, -6.22)
        point_conversion_nww: (82.73, -129.34)
        point_conversion_se: (-60.35, 13.53)
        point_conversion_see: (-37.62, 171.82)
        point_conversion_sw: (-79.83, -83.25)
        point_conversion_sww: (-45.84, -179.22)
    }

    macro_rules! vincenty_inverse {
        (($lat1:expr, $lon1:expr),($lat2:expr, $lon2:expr)) => {
            vincenty_inverse(
                Point::new($lat1, $lon1),
                Point::new($lat2, $lon2),
                100,
                1e-12,
            )
        };
    }
    macro_rules! vincenty_inverse_tests {
        ($($name:ident: ($lat1:expr, $lon1:expr) ($lat2:expr, $lon2:expr) $exp:expr)*) => {
        $(
            #[test]
            fn $name() {
                assert_abs_diff_eq!(
                    vincenty_inverse!(($lat1, $lon1), ($lat2, $lon2)).unwrap(),
                    $exp,
                    epsilon = 1.0e-2
                );
            }
        )*
        }
    }
    vincenty_inverse_tests! {
        vincenty_inverse_short: (48.154563, 17.072561) (48.154564, 17.072562) 0.13378944117648012
        vincenty_inverse_medium: (48.154563, 17.072561) (48.158800, 17.064064) 788.41482952369672
        vincenty_inverse_long: (48.148636, 17.107558) (48.208810, 16.372477) 55073.68246366003
        vincenty_inverse_equatorial: (0.0, 0.0) (0.0, 100.0) 11131949.079
        vincenty_inverse_coincident: (12.3, 4.56) (12.3, 4.56) 0.0
    }
    #[test]
    fn vincenty_inverse_antipodal() {
        assert_eq!(vincenty_inverse!((4.0, 2.0), (-4.0, -178.0)), None)
    }

    macro_rules! projection {
        (($lat1:expr, $lon1:expr), ($lat2:expr, $lon2:expr), ($lat3:expr, $lon3:expr)) => {{
            let geodesic = Geodesic::new(Point::new($lat1, $lon1), Point::new($lat2, $lon2));
            let point = Point::new($lat3, $lon3);

            point.project_onto(geodesic)
        }};
    }
    macro_rules! projection_side_tests {
        ($($name:ident: ($lat1:expr, $lon1:expr) ($lat2:expr, $lon2:expr) ($lat3:expr, $lon3:expr) $side:expr)*) => {
        $(
            #[test]
            fn $name() {
                let (_, _, side) = projection!(($lat1, $lon1), ($lat2, $lon2), ($lat3, $lon3));
                assert_eq!(side, $side);
            }
        )*
        }
    }
    projection_side_tests! {
        meridian_projection_right: (45.0, 7.0) (46.0, 7.0) (45.5, 6.5) Side::Left
        meridian_projection_left: (45.0, 7.0) (46.0, 7.0) (45.5, 7.5) Side::Right
        parallel_projection_right: (45.0, 7.0) (45.0, 8.0) (45.5, 7.5) Side::Left
        parallel_projection_left: (45.0, 7.0) (45.0, 8.0) (44.5, 7.5) Side::Right
    }
    macro_rules! projection_sequence_tests {
        ($($name:ident: ($lat1:expr, $lon1:expr) ($lat2:expr, $lon2:expr) ($lat3:expr, $lon3:expr) $seq:expr)*) => {
        $(
            #[test]
            fn $name() {
                let (_, sequence, _) = projection!(($lat1, $lon1), ($lat2, $lon2), ($lat3, $lon3));
                assert_eq!(sequence, $seq);
            }
        )*
        }
    }
    projection_sequence_tests! {
        meridian_projection_before: (45.0, 7.0) (46.0, 7.0) (44.5, 7.0) Sequence::BeforeStart
        meridian_projection_between: (45.0, 7.0) (46.0, 7.0) (45.5, 7.0) Sequence::InBetween
        meridian_projection_after: (45.0, 7.0) (46.0, 7.0) (46.5, 7.0) Sequence::AfterEnd
        parallel_projection_before: (45.0, 7.0) (45.0, 8.0) (45.0, 6.5) Sequence::BeforeStart
        parallel_projection_between: (45.0, 7.0) (45.0, 8.0) (45.0, 7.5) Sequence::InBetween
        parallel_projection_after: (45.0, 7.0) (45.0, 8.0) (45.0, 8.5) Sequence::AfterEnd
    }
}
