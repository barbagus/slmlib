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

use anyhow::{anyhow, bail, Result};
use color_print::cstr;
use slmlib::{self, burdell, files, geowizard, Coordinates};
use std::{env, fs, path::PathBuf};

const USAGE: &str = cstr!(
    "<bold,underline>Usage:</> slm-cli[.exe] [OPTIONS] FILE

<bold,underline>Arguments:</>
  FILE  Input file.

<bold,underline>Options:</>
  -s, --start POINT    Route start point.
  -e, --end POINT      Route end point.
  -f, --format FORMAT  Input file format (default: input file extension).
  -h, --help           Show this message.

<bold,underline>Values:</>
  POINT   Comma separated coordinates (latitude, longitude) as decimal degrees; north and east as
          positive values, south and west as negative values. Ex: '52.606,-1.91787'
  FORMAT  <bold>csv</>: one POINT per line (optional header).
          <bold>gpx</>: first track.
"
);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Format {
    Csv,
    Gpx,
}

fn parse_point(value: &str) -> Result<Coordinates> {
    let (lat, lon) = value.split_once(',').ok_or(anyhow!("No comma found."))?;
    let lat = lat.parse::<f64>()?;
    let lon = lon.parse::<f64>()?;
    Ok(Coordinates {
        latitude: lat,
        longitude: lon,
    })
}

fn main() -> Result<()> {
    let mut start: Option<Coordinates> = None;
    let mut end: Option<Coordinates> = None;
    let mut input_format: Option<Format> = None;
    let mut input_path: Option<PathBuf> = None;

    let mut args = env::args().skip(1);

    loop {
        if let Some(arg) = args.next() {
            match arg.as_str() {
                "-s" | "--start" => {
                    let value = args.next().ok_or(anyhow!(
                        "option {} requires a <POINT> value.\n\n{}",
                        arg,
                        USAGE
                    ))?;
                    start.replace(parse_point(&value)?);
                }
                "-e" | "--end" => {
                    let value = args.next().ok_or(anyhow!(
                        "option {} requires a POINT value.\n\n{}",
                        arg,
                        USAGE
                    ))?;
                    end.replace(parse_point(&value)?);
                }
                "-f" | "--format" => match args
                    .next()
                    .ok_or(anyhow!(
                        "option {} requires a 'csv' or 'gpx' value.\n\n{}",
                        arg,
                        USAGE
                    ))?
                    .as_str()
                {
                    "csv" => {
                        input_format.replace(Format::Csv);
                    }
                    "gpx" => {
                        input_format.replace(Format::Gpx);
                    }
                    token => {
                        bail!("Unsupported input format: {}\n\n{}", token, USAGE);
                    }
                },
                "-h" | "--help" => {
                    println!("{}", USAGE);
                    return Ok(());
                }
                token => {
                    if token.starts_with("-") {
                        bail!("Unsupported option: {}\n\n{}", token, USAGE);
                    }
                    input_path.replace(token.into());
                }
            }
        } else {
            break;
        }
    }

    let input_path = if let Some(input_path) = input_path {
        input_path
    } else {
        bail!("Missing input file.\n\n{}", USAGE);
    };

    let input_format = if let Some(input_format) = input_format {
        input_format
    } else {
        let ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or(anyhow!(
                "Unable to determine input format from file extension; consider '-f' option.\n\n{}",
                USAGE
            ))?;

        match ext {
            "csv" => Format::Csv,
            "gpx" => Format::Gpx,
            _ => bail!(
                "Unsupported file extension '{}'; consider '-f' option.\n\n{}",
                ext,
                USAGE
            ),
        }
    };

    let buf = fs::read(input_path)?;

    let track = match input_format {
        Format::Csv => files::csv::load(&buf)?,
        Format::Gpx => files::gpx::load(&buf)?,
    };

    if track.is_empty() {
        bail!("Track is empty.");
    }

    let start = start.unwrap_or_else(|| track.first().unwrap().clone());
    let end = end.unwrap_or_else(|| track.last().unwrap().clone());

    let stats = slmlib::analyze(start, end, track);
    println!(
        "Route length:             {:.1} km",
        (stats.route_length / 1000_f64)
    );
    println!("Max. deviation:           {:.1} m", stats.max_deviation);

    let medal = geowizard::compute_rank(&stats);
    let medal = medal.map(|r| r.to_str()).unwrap_or("-");
    println!("Medal rank:               {}", medal);

    let burdell_score = burdell::compute_score(burdell::LVL_PRO, &stats);
    println!("Burdell score (PRO):      {:.1} %", burdell_score);
    let burdell_score = burdell::compute_score(burdell::LVL_AMATEUR, &stats);
    println!("Burdell score (AMATEUR):  {:.1} %", burdell_score);
    let burdell_score = burdell::compute_score(burdell::LVL_NEWBIE, &stats);
    println!("Burdell score (NEWBIE):   {:.1} %", burdell_score);

    Ok(())
}
