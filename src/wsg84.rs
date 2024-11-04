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

use crate::{geo::Point, linalg::Vector};

const A: f64 = 6378137.0;
const F: f64 = 1.0 / 298.257223563;
const B: f64 = A * (1.0 - F);

const A2: f64 = A * A;
const B2: f64 = B * B;
const AB: f64 = A * B;

impl Point {
    /// Convert to cartesian coordinates within the WSG84 model.
    pub fn as_wsg84_vector(&self) -> Vector {
        let (sin_lat, cos_lat) = self.lat_rad.sin_cos();
        let (sin_lon, cos_lon) = self.lon_rad.sin_cos();

        let r: f64 = AB / (B2 * cos_lat * cos_lat + A2 * sin_lat * sin_lat).sqrt();

        Vector {
            x: cos_lon * cos_lat * r,
            y: sin_lon * cos_lat * r,
            z: sin_lat * r,
        }
    }
}

impl Vector {
    /// Convert to geographical coordinates within the WSG84 model.
    pub fn as_wsg84_point(&self) -> Point {
        let value = self.to_unit();

        Point {
            lat_rad: value.z.asin(),
            lon_rad: value.y.atan2(value.x),
        }
    }
}

/// Compute the geodetic distance between two points, using Vincenty's formulae.
pub fn vincenty_inverse(p1: Point, p2: Point, max_iteration: i32, accuracy: f64) -> Option<f64> {
    let big_l = p2.lon_rad - p1.lon_rad;

    let (sin_u1, cos_u1) = ((1_f64 - F) * p1.lat_rad.tan()).atan().sin_cos();
    let (sin_u2, cos_u2) = ((1_f64 - F) * p2.lat_rad.tan()).atan().sin_cos();

    let mut lambda = big_l;

    for _ in 0..max_iteration {
        let (sin_lambda, cos_lambda) = lambda.sin_cos();

        let sin_sigma = ((cos_u2 * sin_lambda) * (cos_u2 * sin_lambda)
            + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda)
                * (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda))
            .sqrt();

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

        let sigma = sin_sigma.atan2(cos_sigma);

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

        if (lambda - lambda_prev).abs() <= accuracy {
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
    use approx::assert_abs_diff_eq;

    use super::*;

    macro_rules! conversion_tests {
        ($($name:ident: ($lat:expr,$lon:expr),)*) => {
        $(
            #[test]
            fn $name() {
                let p = Point::new($lat, $lon).unwrap();
                let q = p.as_wsg84_vector().as_wsg84_point();
                assert!(p.approx_eq(q, 1e-12))
            }
        )*
        }
    }

    conversion_tests! {
        conversion_ne: (78.29, 21.83),
        conversion_nee: (56.15, 145.18),
        conversion_nw: (47.21, -6.22),
        conversion_nww: (82.73, -129.34),
        conversion_se: (-60.35, 13.53),
        conversion_see: (-37.62, 171.82),
        conversion_sw: (-79.83, -83.25),
        conversion_sww: (-45.84, -179.22),
    }

    macro_rules! vincenty_inverse {
        (($lat1:expr, $lon1:expr),($lat2:expr, $lon2:expr)) => {
            vincenty_inverse(
                Point::new($lat1, $lon1).unwrap(),
                Point::new($lat2, $lon2).unwrap(),
                100,
                1e-12,
            )
        };
    }

    #[test]
    fn vincenty_inverse_short() {
        assert_abs_diff_eq!(
            vincenty_inverse!((48.154563, 17.072561), (48.154564, 17.072562)).unwrap(),
            0.13378944117648012,
            epsilon = 1.0e-2
        );
    }

    #[test]
    fn vincenty_inverse_medium() {
        assert_abs_diff_eq!(
            vincenty_inverse!((48.154563, 17.072561), (48.158800, 17.064064)).unwrap(),
            788.4148295236967,
            epsilon = 1.0e-2
        );
    }

    #[test]
    fn vincenty_inverse_long() {
        assert_abs_diff_eq!(
            vincenty_inverse!((48.148636, 17.107558), (48.208810, 16.372477)).unwrap(),
            55073.68246366003,
            epsilon = 1.0e-2
        );
    }

    #[test]
    fn vincenty_inverse_equatorial() {
        assert_abs_diff_eq!(
            vincenty_inverse!((0.0, 0.0), (0.0, 100.0)).unwrap(),
            11131949.079,
            epsilon = 1.0e-2
        );
    }

    #[test]
    fn vincenty_inverse_coincident() {
        assert_abs_diff_eq!(
            vincenty_inverse!((12.3, 4.56), (12.3, 4.56)).unwrap(),
            0.0,
            epsilon = 1.0e-2
        );
    }

    #[test]
    fn vincenty_inverse_antipodal() {
        assert_eq!(vincenty_inverse!((4.0, 2.0), (-4.0, -178.0)), None)
    }
}
