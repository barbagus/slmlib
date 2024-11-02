use crate::stats::Stats;
use core::iter;

/// A Burdell score penalty setting.
#[derive(Debug, Clone, Copy)]
pub struct BurdellSettings {
    /// Distance in meters over which each penalty terms computed (small is more severe).
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

pub fn compute(config: BurdellSettings, stats: &Stats) -> f64 {
    let steps = (stats.target_line_length / config.step).ceil() as usize;

    let mut slots: Vec<Option<f64>> = vec![None; steps];

    let mut used_slots: Vec<usize> = Vec::with_capacity(slots.len());

    for p in stats.points.iter() {
        let i = (p.made_good / config.step).floor() as usize;
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

    for (i1, i2) in iter::zip(
        used_slots.iter().cloned(),
        used_slots.iter().skip(1).cloned(),
    ) {
        if i2 - i1 > 1 {
            let fill = (slots[i1].unwrap() + slots[i2].unwrap()) / 2.0;
            for i in (i1 + 1)..i2 {
                slots[i].replace(fill);
            }
        }
    }

    for i in 0..(*used_slots.first().unwrap()) {
        slots[i].replace(0_f64);
    }
    for i in (*used_slots.last().unwrap() + 1)..slots.len() {
        slots[i].replace(0_f64);
    }

    let log = stats.target_line_length.log10();
    let mut penalities: f64 = 0.0;
    for slot in slots {
        penalities += 100.0 * (slot.unwrap() / config.coefficient).powf(log);
    }

    f64::max(100.0 - penalities, 0.0)
}
