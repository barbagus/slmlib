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

use super::geo::Point;
use std::{
    borrow::Cow,
    error,
    fmt::Display,
    io::{self, Read},
    str,
};
use xml::{
    common::Position,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug)]
pub enum ErrorKind {
    Eof,
    Io(io::Error),
    MissingLatitude,
    MissingLongitude,
    Utf8(str::Utf8Error),
    Value,
    Xml(Cow<'static, str>),
}

#[derive(Debug)]
pub struct Error {
    pub row: u64,
    pub column: u64,
    pub kind: ErrorKind,
}

impl From<xml::reader::Error> for Error {
    fn from(err: xml::reader::Error) -> Self {
        let position = err.position();

        let kind = match err.kind().to_owned() {
            xml::reader::ErrorKind::Syntax(cow) => ErrorKind::Xml(cow),
            xml::reader::ErrorKind::Io(error) => ErrorKind::Io(error),
            xml::reader::ErrorKind::Utf8(utf8_error) => ErrorKind::Utf8(utf8_error),
            xml::reader::ErrorKind::UnexpectedEof => ErrorKind::Eof,
        };

        Self {
            row: position.row,
            column: position.column,
            kind,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} ", self.row, self.column)?;
        match self.kind {
            ErrorKind::Eof => write!(f, "unexpected EOF"),
            ErrorKind::Io(ref error) => error.fmt(f),
            ErrorKind::MissingLatitude => write!(f, "missing latitude"),
            ErrorKind::MissingLongitude => write!(f, "missing longitude"),
            ErrorKind::Utf8(ref utf8_error) => utf8_error.fmt(f),
            ErrorKind::Value => write!(f, "ill-formed coordinate value"),
            ErrorKind::Xml(ref cow) => f.write_str(cow),
        }
    }
}

impl error::Error for Error {}

pub struct GpxReader<R: Read> {
    inner: EventReader<R>,
    has_finished: bool,
}

impl<R: Read> GpxReader<R> {
    pub fn new(rdr: R) -> Self {
        Self {
            inner: EventReader::new(rdr),
            has_finished: false,
        }
    }

    pub fn get_ref(&self) -> &R {
        &self.inner.source()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.inner.source_mut()
    }

    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }

    fn fail(&mut self, kind: ErrorKind) -> Option<Result<Point, Error>> {
        self.has_finished = true;
        let position = self.inner.position();
        return Some(Err(Error {
            row: position.row + 1,
            column: position.column + 1,
            kind,
        }));
    }
}

impl<R: Read> Iterator for GpxReader<R> {
    type Item = Result<Point, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_finished {
            return None;
        }

        loop {
            let event = match self.inner.next() {
                Ok(event) => event,
                Err(err) => {
                    self.has_finished = true;
                    return Some(Err(err.into()));
                }
            };

            match event {
                XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "trkpt" => {
                    let mut lat: Option<f64> = None;
                    let mut lon: Option<f64> = None;
                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            "lat" => match attr.value.parse::<f64>() {
                                Err(_) => return self.fail(ErrorKind::Value),
                                Ok(value) => {
                                    lat.replace(value);
                                }
                            },
                            "lon" => match attr.value.parse::<f64>() {
                                Err(_) => return self.fail(ErrorKind::Value),
                                Ok(value) => {
                                    lon.replace(value);
                                }
                            },
                            _ => {
                                continue;
                            }
                        }
                    }

                    let lat = match lat {
                        Some(lat) => lat,
                        None => return self.fail(ErrorKind::MissingLatitude),
                    };

                    let lon = match lon {
                        Some(lon) => lon,
                        None => return self.fail(ErrorKind::MissingLongitude),
                    };

                    return Some(Ok(Point::new(lat, lon)));
                }
                XmlEvent::EndElement { name } if name.local_name == "trk" => {
                    self.has_finished = true;
                    return None;
                }
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! gpx_tests_ok {
        ($($name:ident: $csv:literal,)*) => {
        $(
            #[test]
            fn $name() {
                let rdr = GpxReader::new($csv.as_bytes());
                let points = rdr.map(|r| r.unwrap()).collect::<Vec<_>>();
                let check = vec![
                    Point::new(47.6655080, 8.5671500),
                    Point::new(47.6655040, 8.5671580),
                    Point::new(47.6655010, 8.5671610),
                ]
                .into_iter()
                .collect::<Vec<_>>();

                assert_eq!(points, check);
            }
        )*
        }
    }
    gpx_tests_ok! {
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
