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

//! A straight line mission track as a CSV file. Specifically:
//!  - Each line of the file should have only one latitude/longitude pair (separated by a comma).
//!  - Latitudes and longitudes should be expressed in decimal degrees.
//!  - Degrees North and East should be expressed as positive values.
//!  - Degrees South and West should be expressed as negative values.
//!  - A header row is optional
//!  - File should be UTF-8 encoded
//!
//! Example:
//! ```csv
//!Latitude, Longitude
//!52.6060,-1.91786
//!52.6061,-1.91787
//!52.6062,-1.91788
//! ```
//!
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{
    error,
    fmt::{self, Write},
    num, str, u64,
};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Overflow,
    Syntax,
    Utf8,
    Value,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub row: u64,
    pub column: u64,
    pub kind: ErrorKind,
}

const ERR_OVERFLOW: Error = Error {
    row: u64::MAX,
    column: u64::MAX,
    kind: ErrorKind::Overflow,
};

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} ", self.row, self.column)?;
        match self.kind {
            ErrorKind::Overflow => f.write_str("row/line too large"),
            ErrorKind::Syntax => f.write_str("no comma separator"),
            ErrorKind::Utf8 => f.write_str("invalid utf-8 encoding"),
            ErrorKind::Value => f.write_str("ill-formed coordinate value"),
        }
    }
}

impl error::Error for Error {}

impl From<num::TryFromIntError> for Error {
    fn from(_: num::TryFromIntError) -> Self {
        ERR_OVERFLOW.clone()
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Self {
        ERR_OVERFLOW.clone()
    }
}

const LINE_SEP: u8 = b'\n';
const FIELD_SEP: &str = ",";

macro_rules! inc {
    ($i:expr) => {{
        let row: u64 = $i.try_into()?;
        row.checked_add(1).ok_or(ERR_OVERFLOW.clone())?
    }};
}

pub fn load(buf: &[u8]) -> Result<Vec<(f64, f64)>, Error> {
    let buf = if let Some(b) = buf.last() {
        if b == &LINE_SEP {
            &buf[..(buf.len() - 1)]
        } else {
            buf
        }
    } else {
        return Ok(Vec::new());
    };

    let capacity: usize = buf.iter().filter(|c| *c == &LINE_SEP).count() + 1;

    let mut track: Vec<(f64, f64)> = Vec::with_capacity(capacity);

    for (i, line) in buf.split(|c| c == &LINE_SEP).enumerate() {
        let line = match str::from_utf8(line) {
            Ok(line) => line,
            Err(_) => {
                return Err(Error {
                    row: inc!(i),
                    column: 0,
                    kind: ErrorKind::Utf8,
                });
            }
        };

        let (lhs, rhs) = line.split_once(FIELD_SEP).ok_or(Error {
            row: inc!(i),
            column: 0,
            kind: ErrorKind::Syntax,
        })?;

        let lat = match lhs.trim().parse::<f64>() {
            Ok(lat) => lat,
            Err(_) => {
                if i == 0 {
                    // optional header
                    continue;
                } else {
                    return Err(Error {
                        row: inc!(i),
                        column: 0,
                        kind: ErrorKind::Value,
                    });
                }
            }
        };

        let lon = match rhs.trim().parse::<f64>() {
            Ok(lon) => lon,
            Err(_) => {
                return Err(Error {
                    row: inc!(i),
                    column: inc!(lhs.len()),
                    kind: ErrorKind::Value,
                });
            }
        };

        track.push((lat, lon));
    }

    Ok(track)
}

pub fn dump<I: IntoIterator<Item = (f64, f64)>>(track: I) -> Result<Vec<u8>, Error> {
    let mut csv = String::new();

    writeln!(&mut csv, "Latitude,Longitude")?;
    for (lat, lon) in track {
        writeln!(&mut csv, "{:.8},{:.8}", lat, lon)?;
    }
    Ok(csv.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "Latitude,Longitude
54.29600470,-4.58877725
54.29600654,-4.58877590
54.29600906,-4.58876509
";

    macro_rules! csv_load_ok_tests {
            ($($name:ident: $csv:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let track = load($csv.as_bytes()).unwrap();
                    let check = alloc::vec![
                        (54.29600470, -4.58877725),
                        (54.29600654, -4.58877590),
                        (54.29600906, -4.58876509),
                    ]
                    .into_iter()
                    .collect::<Vec<_>>();

                    assert_eq!(track, check);
                }
            )*
            }
        }
    csv_load_ok_tests! {
        load_with_happy: CSV,
        load_with_headers: "Latitude,Longitude
54.29600470,-4.58877725
54.29600654,-4.58877590
54.29600906,-4.58876509",
        load_without_headers: "54.29600470,-4.58877725
54.29600654,-4.58877590
54.29600906,-4.58876509",
    }

    #[test]
    fn dump_happy() {
        let track = [
            (54.29600470, -4.58877725),
            (54.29600654, -4.58877590),
            (54.29600906, -4.58876509),
        ];
        let csv = dump(track).unwrap();
        let csv = str::from_utf8(&csv).unwrap();
        assert_eq!(csv, CSV);
    }
}
