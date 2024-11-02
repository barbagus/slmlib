use crate::{geo::Point, wsg84::vincenty_inverse};

#[derive(Debug, Clone)]
pub struct PointStats {
    pub deviation: f64,
    pub made_good: f64,
}

pub struct Stats {
    pub target_line_length: f64,
    pub max_deviation: f64,
    pub points: Vec<PointStats>,
}

pub fn geodetic_distance(p1: Point, p2: Point) -> Option<f64> {
    vincenty_inverse(p1, p2, 100, 1e-9)
}

/// Compute statistics for the line and every given points
pub fn compute<I>(target_line: (Point, Point), points: I) -> Stats
where
    I: IntoIterator<Item = Point>,
{
    let (start, end) = target_line;

    let target_line_length = geodetic_distance(start, end).unwrap();

    let v_start = start.as_wsg84_vector();
    let v_end = end.as_wsg84_vector();
    let v_normal = v_end.cross(v_start).to_unit();

    let mut max_deviation = 0_f64;

    let points = points
        .into_iter()
        .map(|point| {
            let v_point = point.as_wsg84_vector();

            let f = v_point.dot(v_normal);
            let v_projection = v_point.sub(v_normal.mul(f));
            let projection = v_projection.as_wsg84_point();

            let deviation = geodetic_distance(point, projection).unwrap();

            let h = v_projection.cross(v_start).dot(v_normal);
            let made_good = geodetic_distance(start, projection).unwrap() * h.signum();

            // questionable cap of track before start and after end.
            let ps = if made_good < 0_f64 {
                PointStats {
                    deviation: 0_f64,
                    made_good: 0_f64,
                }
            } else if made_good > target_line_length {
                PointStats {
                    deviation: 0_f64,
                    made_good: target_line_length,
                }
            } else {
                PointStats {
                    deviation,
                    made_good,
                }
            };

            if ps.deviation > max_deviation {
                max_deviation = ps.deviation;
            }

            ps
        })
        .collect();

    Stats {
        target_line_length,
        max_deviation,
        points,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::files::sml::SMLDoc;
    use approx::assert_abs_diff_eq;
    use serde_json;

    macro_rules! mission_tests {
        ($($name:ident: $sml_path:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let rdr = std::fs::File::open($sml_path).unwrap();
                let sml = serde_json::from_reader::<_, SMLDoc>(rdr).unwrap().attempt;

                let stats = compute(sml.target_line(), sml.geo_points());

                assert_abs_diff_eq!(
                    stats.target_line_length,
                    sml.target_line_length,
                    epsilon = 1e-2
                );

                for (point_stats, sml_point) in std::iter::zip(stats.points.iter(), sml.points.iter()) {
                    assert_abs_diff_eq!(
                        point_stats.deviation,
                        sml_point.distance_to_line,
                        epsilon = 1e-2,
                    );
                    assert_abs_diff_eq!(
                        point_stats.made_good,
                        sml_point.control_point_distance_to_start,
                        epsilon = 1e-2
                    );
                }
            }
        )*
        }
    }

    mission_tests! {
        mission_archie_iom: "fixtures/archie-iom.sml",
        mission_archie_scotland: "fixtures/archie-scotland.sml",
        mission_archie_wales_run: "fixtures/archie-wales-run.sml",
        mission_archie_wales_walk: "fixtures/archie-wales-walk.sml",
        mission_geowizard_iom: "fixtures/geowizard-iom.sml",
        mission_geowizard_norway: "fixtures/geowizard-norway.sml",
        mission_geowizard_scotland: "fixtures/geowizard-scotland.sml",
        mission_geowizard_wales1a: "fixtures/geowizard-wales1a.sml",
        mission_geowizard_wales1b: "fixtures/geowizard-wales1b.sml",
        mission_geowizard_wales2: "fixtures/geowizard-wales2.sml",
        mission_geowizard_wales3: "fixtures/geowizard-wales3.sml",
        mission_geowizard_wales4: "fixtures/geowizard-wales4.sml",
        mission_hiiumaa: "fixtures/hiiumaa.sml",
        mission_muhu: "fixtures/muhu.sml",
        mission_new_forest: "fixtures/new-forest.sml",
        mission_saaremaa: "fixtures/saaremaa.sml",
        mission_schaffhausen: "fixtures/schaffhausen.sml",
    }
}
