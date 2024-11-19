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
//! "meta" files.

use dev::sml;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() {
    let input_file: PathBuf = env::args().nth(1).expect("No input file specified.").into();
    let output_file = input_file.clone().with_extension("csv");

    let attempt = sml::load(input_file);
    let wtr = File::create(output_file).expect("Open output file");
    let mut wtr = BufWriter::new(wtr);
    writeln!(&mut wtr, "Latitude,Longitude").expect("Write headers");
    for point in attempt.points {
        writeln!(&mut wtr, "{:.6},{:.6}", point.latitude, point.longitude).expect("Write record");
    }
}
