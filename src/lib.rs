pub use burdell::{BurdellSettings, LVL_AMATEUR, LVL_NEWBIE, LVL_PRO};
pub use geo::Point;
pub use stats::geodetic_distance;

mod burdell;
mod geo;
mod linalg;
mod stats;
mod wsg84;

pub mod files;

/// A scoring configuration
#[derive(Debug)]
pub struct Config {
    /// Burdell penalty setting.
    pub burdell: BurdellSettings,
}

impl Default for Config {
    fn default() -> Self {
        Self { burdell: LVL_PRO }
    }
}

/// The medal color associated with a max deviation value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Medal {
    /// Max deviation less than 25 meters.
    Platinum,
    /// Max deviation less than 50 meters.
    Gold,
    /// Max deviation less than 75 meters.
    Silver,
    /// Max deviation less than 100 meters.
    Bronze,
}

impl Medal {
    pub fn from_max_deviation(value: f64) -> Option<Self> {
        let value = value.abs();
        if value < 25.0 {
            Some(Medal::Platinum)
        } else if value < 50.0 {
            Some(Medal::Gold)
        } else if value < 75.0 {
            Some(Medal::Silver)
        } else if value < 100.0 {
            Some(Medal::Bronze)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Score {
    pub target_line_length: f64,
    pub max_deviation: f64,
    pub medal: Option<Medal>,
    pub burdell: f64,
}

pub fn score_my_line<I>(config: Config, target_line: (Point, Point), points: I) -> Score
where
    I: IntoIterator<Item = Point>,
{
    let stats = stats::compute(target_line, points);

    Score {
        target_line_length: stats.target_line_length,
        max_deviation: stats.max_deviation,
        medal: Medal::from_max_deviation(stats.max_deviation),
        burdell: burdell::compute(config.burdell, &stats),
    }
}
