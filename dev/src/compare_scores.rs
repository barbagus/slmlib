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

//! A tool to compute and display the relative correctness of our implementation with regards to the
//! original implementation over at [scoremyline.com](https://scoremyline.com)
//!
//! The expected results have been manually collected from the site and organized in so called
//! "fix" files.

use slmlib::{compute_stats, Point};
use std::fs;

fn fmt_err(err: f64) -> String {
    let s = format!("{:.2}", err);

    let mut precision: Option<usize> = None;

    for (i, c) in s.chars().rev().enumerate() {
        if c == '.' || c == '-' || c == '0' {
            continue;
        }
        precision.replace(i + 1);
    }

    match precision {
        None => String::from("-"),
        Some(index) => match index {
            0 => panic!(),
            1 => format!("{}", s),
            2 => format!("*{}*", s),
            _ => format!("**{}**", s),
        },
    }
}

fn main() {
    let mut paths = fs::read_dir("../fixtures")
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.metadata().unwrap().is_file())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sml"))
        .collect::<Vec<_>>();
    paths.sort();

    println!(
        "| {} |",
        [
            "Mission           ",
            "Pro         ",
            "Pro err.    ",
            "Amateur     ",
            "Amateur err.",
            "Newbie      ",
            "Newbie err. ",
        ]
        .join(" | ")
    );
    println!(
        "|{}|",
        [
            ":-------------------",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
            "-------------:",
        ]
        .join("|")
    );

    for sml_path in paths {
        let name = sml_path
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        let fix_path = sml_path.clone().with_extension("json");

        let (route, track) = {
            let attempt = {
                let buf = fs::read(sml_path).expect("read SML file");
                files::sml::load(&buf).expect("load SML file")
            };
            let route = {
                let (start, end) = attempt.route();
                let start = Point::new(start.0, start.1);
                let end = Point::new(end.0, end.1);
                (start, end)
            };

            (
                route,
                attempt
                    .track()
                    .map(|t| Point::new(t.0, t.1))
                    .collect::<Vec<_>>(),
            )
        };

        let fix = {
            let buf = fs::read(fix_path).expect("read FIX file");
            files::fix::load(&buf).expect("parse FIX file")
        };

        for score in fix.scores.into_iter().filter(|s| s.ignore.is_none()) {
            let stats = compute_stats(route, track.iter().cloned());
            let pro = {
                let burdell_score = slmlib::compute_burdell_score(slmlib::LVL_PRO, &stats);
                (burdell_score, burdell_score - score.scores.pro)
            };
            let amateur = {
                let burdell_score = slmlib::compute_burdell_score(slmlib::LVL_AMATEUR, &stats);
                (burdell_score, burdell_score - score.scores.amateur)
            };
            let newbie = {
                let burdell_score = slmlib::compute_burdell_score(slmlib::LVL_NEWBIE, &stats);
                (burdell_score, burdell_score - score.scores.newbie)
            };

            println!(
                "| {} |",
                [
                    format!("{:18}", name),
                    format!("{:12.2}", pro.0),
                    format!("{:>12}", fmt_err(pro.1)),
                    format!("{:12.2}", amateur.0),
                    format!("{:>12}", fmt_err(amateur.1)),
                    format!("{:12.2}", newbie.0),
                    format!("{:>12}", fmt_err(newbie.1)),
                ]
                .join(" | ")
            );
        }
    }
}
