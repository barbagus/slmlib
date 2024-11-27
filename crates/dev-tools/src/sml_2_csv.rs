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

//! A tool to convert a Score My Line (SML) file to CSV.

use slmlib::Coordinates;
use std::{
    env,
    fmt::{Error, Write},
    fs,
    path::PathBuf,
};

pub fn dump<I: IntoIterator<Item = Coordinates>>(track: I) -> Result<Vec<u8>, Error> {
    let mut csv = String::new();

    writeln!(&mut csv, "Latitude,Longitude")?;
    for Coordinates {
        latitude,
        longitude,
    } in track
    {
        writeln!(&mut csv, "{:.8},{:.8}", latitude, longitude)?;
    }
    Ok(csv.into())
}

fn main() {
    let input_path: PathBuf = env::args().nth(1).expect("no input file specified").into();
    let output_path = input_path.clone().with_extension("csv");

    let buf = fs::read(input_path).expect("read input file");
    let attempt = slmlib::files::sml::load(&buf).expect("load SML file");

    let buf = dump(attempt.track()).expect("dump CSV file");
    fs::write(output_path, buf).expect("write CSV file");
}
