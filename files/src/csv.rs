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

use slmlib::Point;
use std::{
    error,
    fmt::Display,
    io::{self, BufRead, BufReader, Read},
    str,
};

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    NoComa,
    Overflow,
    Utf8(str::Utf8Error),
    Value,
}

#[derive(Debug)]
pub struct Error {
    pub row: u64,
    pub column: u64,
    pub kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} ", self.row, self.column)?;
        match self.kind {
            ErrorKind::Io(ref error) => error.fmt(f),
            ErrorKind::NoComa => write!(f, "no comma separator"),
            ErrorKind::Overflow => write!(f, "row/line too large"),
            ErrorKind::Utf8(ref utf8_error) => utf8_error.fmt(f),
            ErrorKind::Value => write!(f, "ill-formed coordinate value"),
        }
    }
}

impl error::Error for Error {}

pub struct CsvReader<R: Read> {
    row: u64,
    column: u64,
    line_buffer: Vec<u8>,
    has_failed: bool,
    inner: BufReader<R>,
}

impl<R: Read> CsvReader<R> {
    pub fn new(rdr: R) -> Self {
        Self {
            row: 0,
            column: 0,
            has_failed: false,
            inner: BufReader::new(rdr),
            line_buffer: Vec::new(),
        }
    }

    pub fn with_capacity(rdr: R, capacity: usize) -> Self {
        Self {
            row: 0,
            column: 0,
            has_failed: false,
            inner: BufReader::new(rdr),
            line_buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn get_ref(&self) -> &R {
        &self.inner.get_ref()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }

    fn fail(&mut self, kind: ErrorKind) -> Option<Result<Point, Error>> {
        self.has_failed = true;
        Some(Err(Error {
            row: self.row,
            column: self.column,
            kind,
        }))
    }
}

impl<R: Read> Iterator for CsvReader<R> {
    type Item = Result<Point, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_failed {
            return None;
        }

        self.row = match self.row.checked_add(1) {
            Some(row) => row,
            None => return self.fail(ErrorKind::Overflow),
        };

        self.column = 1;

        match self.inner.read_until(b'\n', &mut self.line_buffer) {
            Ok(0) => return None,
            Ok(_) => {}
            Err(err) => return self.fail(ErrorKind::Io(err)),
        }

        let line = match str::from_utf8(&self.line_buffer) {
            Ok(line) => line.trim_end(),
            Err(err) => return self.fail(ErrorKind::Utf8(err)),
        };

        let (lat, lon) = match line.split_once(',') {
            Some(fields) => fields,
            None => return self.fail(ErrorKind::NoComa),
        };
        let offset = lat.len();

        let lat = match lat.parse::<f64>() {
            Ok(lat) => lat,
            Err(_) => {
                return if self.row == 1 {
                    self.line_buffer.clear();
                    self.next()
                } else {
                    self.fail(ErrorKind::Value)
                }
            }
        };
        self.column = match offset.try_into() {
            Ok(column) => column,
            Err(_) => return self.fail(ErrorKind::Overflow),
        };

        let lon = match lon.parse::<f64>() {
            Ok(lon) => lon,
            Err(_) => return self.fail(ErrorKind::Value),
        };

        self.line_buffer.clear();

        Some(Ok(Point::new(lat, lon)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! csv_tests_ok {
        ($($name:ident: $csv:literal,)*) => {
        $(
            #[test]
            fn $name() {
                let rdr = CsvReader::new($csv.as_bytes());
                let points = rdr.map(|r| r.unwrap()).collect::<Vec<_>>();
                let check = vec![
                    Point::new(54.296005, -4.588777),
                    Point::new(54.296007, -4.588776),
                    Point::new(54.296009, -4.588765),
                ]
                .into_iter()
                .collect::<Vec<_>>();

                assert_eq!(points, check);
            }
        )*
        }
    }
    csv_tests_ok! {
        with_headers: "Latitude,Longitude
54.296005,-4.588777
54.296007,-4.588776
54.296009,-4.588765",
        without_headers: "54.296005,-4.588777
54.296007,-4.588776
54.296009,-4.588765",
        with_trailing_new_line: "Latitude,Longitude
54.296005,-4.588777
54.296007,-4.588776
54.296009,-4.588765
",
    }
}
