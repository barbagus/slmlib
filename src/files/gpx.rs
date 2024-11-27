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

extern crate alloc;

use crate::Coordinates;
use alloc::vec::Vec;
use core::{error, f64, fmt, str};
use xmlparser::{ElementEnd, TextPos, Token, Tokenizer};

#[derive(Debug, Clone)]
pub enum Error {
    DuplicateCoordinate(TextPos),
    InvalidCoordinate(TextPos),
    MissingCoordinate(TextPos),
    Utf8(str::Utf8Error),
    XmlForm(TextPos),
    XmlStack(TextPos),
    XmlStream(xmlparser::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DuplicateCoordinate(text_pos) => {
                write!(f, "duplicate coordinate at {}", text_pos)
            }
            Error::InvalidCoordinate(text_pos) => {
                write!(f, "invalid coordinate at {}", text_pos)
            }
            Error::MissingCoordinate(text_pos) => {
                write!(f, "missing coordinate at {}", text_pos)
            }
            Error::Utf8(utf8_error) => utf8_error.fmt(f),
            Error::XmlForm(text_pos) => {
                write!(f, "malformed xml at {}", text_pos)
            }
            Error::XmlStack(text_pos) => {
                write!(f, "xml stack at {} (you found a bug !)", text_pos)
            }
            Error::XmlStream(error) => error.fmt(f),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(value: str::Utf8Error) -> Self {
        Error::Utf8(value)
    }
}
impl From<xmlparser::Error> for Error {
    fn from(value: xmlparser::Error) -> Self {
        Self::XmlStream(value)
    }
}

macro_rules! stack_error {
    ($tokenizer: expr) => {
        Error::XmlStack($tokenizer.stream().gen_text_pos())
    };
}

macro_rules! form_error {
    ($tokenizer: expr) => {
        Err(Error::XmlForm($tokenizer.stream().gen_text_pos()))
    };
}

macro_rules! set_coordinate {
    ($tokenizer: expr, $src:expr, $dst: expr) => {{
        let value = $src
            .parse::<f64>()
            .or_else(|_| Err(Error::InvalidCoordinate($tokenizer.stream().gen_text_pos())))?;

        if let Some(_) = $dst.replace(value) {
            return Err(Error::DuplicateCoordinate(
                $tokenizer.stream().gen_text_pos(),
            ));
        };
    }};
}

pub fn load(buf: &[u8]) -> Result<Vec<Coordinates>, Error> {
    let buf = str::from_utf8(buf)?;

    let mut track: Vec<Coordinates> = Vec::new();
    let mut stack: Vec<&str> = Vec::with_capacity(10);

    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;

    let mut tokenizer = Tokenizer::from(buf);
    while let Some(token) = tokenizer.next() {
        let token = token?;

        match token {
            Token::ElementStart { local, .. } => {
                stack.push(local.as_str());
            }
            Token::Attribute { local, value, .. } => {
                match stack.last().ok_or_else(|| stack_error!(tokenizer))? {
                    &"trkpt" => match local.as_str() {
                        "lat" => set_coordinate!(tokenizer, value, lat),
                        "lon" => set_coordinate!(tokenizer, value, lon),
                        _ => continue,
                    },
                    _ => continue,
                }
            }
            Token::ElementEnd { end, .. } => {
                let element = match end {
                    ElementEnd::Open => continue,
                    ElementEnd::Close(_, local) => {
                        let element = stack.pop().ok_or_else(|| stack_error!(tokenizer))?;
                        if local != element {
                            return form_error!(tokenizer);
                        }
                        element
                    }
                    ElementEnd::Empty => stack.pop().ok_or_else(|| stack_error!(tokenizer))?,
                };

                match element {
                    "trk" => break,
                    "trkpt" => {
                        track.push(Coordinates {
                            latitude: lat.take().ok_or_else(|| {
                                Error::MissingCoordinate(tokenizer.stream().gen_text_pos())
                            })?,
                            longitude: lon.take().ok_or_else(|| {
                                Error::MissingCoordinate(tokenizer.stream().gen_text_pos())
                            })?,
                        });
                    }
                    _ => continue,
                }
            }
            _ => continue,
        }
    }
    Ok(track)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! csv_load_ok_tests {
        ($($name:ident: $csv:literal,)*) => {
        $(
            #[test]
            fn $name() {
                let points = load($csv.as_bytes()).unwrap();
                let check = alloc::vec![
                    Coordinates{latitude: 47.6655080, longitude: 8.5671500},
                    Coordinates{latitude: 47.6655040, longitude: 8.5671580},
                    Coordinates{latitude: 47.6655010, longitude: 8.5671610},
                ]
                .into_iter()
                .collect::<Vec<_>>();

                assert_eq!(points, check);
            }
        )*
        }
    }
    csv_load_ok_tests! {
        happy_case: r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd" version="1.1" xmlns="http://www.topografix.com/GPX/1/1">
 <trk>
  <name>Schaffhausen</name>
  <trkseg>
   <trkpt lat="47.6655080" lon="8.5671500" />
   <trkpt lat="47.6655040" lon="8.5671580" />
   <trkpt lat="47.6655010" lon="8.5671610" />
  </trkseg>
 </trk>
</gpx>
"#,
        no_ns: r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx>
 <trk>
  <name>Schaffhausen</name>
  <trkseg>
   <trkpt lat="47.6655080" lon="8.5671500" />
   <trkpt lat="47.6655040" lon="8.5671580" />
   <trkpt lat="47.6655010" lon="8.5671610" />
  </trkseg>
 </trk>
</gpx>
"#,
        multiple_segments: r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx>
 <trk>
  <name>Schaffhausen</name>
  <trkseg>
   <trkpt lat="47.6655080" lon="8.5671500" />
  </trkseg>
  <trkseg>
   <trkpt lat="47.6655040" lon="8.5671580" />
   <trkpt lat="47.6655010" lon="8.5671610" />
  </trkseg>
 </trk>
</gpx>
"#,
        multiple_tracks: r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx>
 <trk>
  <name>Schaffhausen</name>
  <trkseg>
   <trkpt lat="47.6655080" lon="8.5671500" />
   <trkpt lat="47.6655040" lon="8.5671580" />
   <trkpt lat="47.6655010" lon="8.5671610" />
  </trkseg>
 </trk>
 <trk>
  <name>2nd track</name>
  <trkseg>
   <trkpt lat="-47.6655080" lon="-8.5671500" />
   <trkpt lat="-47.6655040" lon="-8.5671580" />
   <trkpt lat="-47.6655010" lon="-8.5671610" />
  </trkseg>
 </trk>
</gpx>
"#,
    }
}
