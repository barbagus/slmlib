# slmlib
Straight line missions utilities

## Objective
Provide a collection of open-source utilities for [straight line missions](https://en.wikipedia.org/wiki/GeoWizard#Straight_line_missions)
(attempts to cross a geographic area in a straight line) enthusiasts.

## Rationale
At this date, the *go to* online utility seems to be the [scoremyline.com](https://scoremyline.com/)
website. Given a *target line* and GPS trace, it provides basic stats (target line length, maximum
deviation), GeoWizard's medal ranking (platinum, gold, silver and bronze) as well as a scoring
scheme known to the community as the *Burdell score* (named after the author of that website).
However, the mathematics and algorithms involved in such computations are:
  1. not particularly well specified
  2. not trivial,
  3. not open source.

The third point implies in particular that:
  1. details and correctness of implementations cannot be verified,
  2. community discussions about these *de facto* standards are difficult to organize.

*Disclaimer*: we have **not** been in contact with Mr. Burdell for lack of means to do so (email, social
media, etc.). But we would welcome the opportunity.

## Implementation
The first efforts are to replicate the results of [scoremyline](https://scoremyline.com/). The bulk
of the sample data have been retrieved from the website itself, but also, shout out to
[the person](https://github.com/SimonJoelWarkentin) behind
[Straight Line Wiki](https://straightline.wiki/) for providing us with additional traces.

### Distances and deviations
Measuring distances on earth is a [tricky business](https://en.wikipedia.org/wiki/Geographical_distance)
and it all comes down to the level of precision that you aim for. Thankfully it is an old and very
well studied problem. We have [implemented](./src/geo.rs) the very popular [Vincenty](https://en.wikipedia.org/wiki/Vincenty%27s_formulae)
*inverse problem* formula on the [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System#WGS84)
[ellipsoidal](https://en.wikipedia.org/wiki/Spheroid) model. We chose the level of precision to be
accurate to the centimeter compared to more involved methods (well beyond the precision of GPS
devices).

Deviations are less trivial to compute as there is no algebraic or numerical solution to the problem
on an ellipsoid model. Our [implementation](./src/geo.rs) assumes (incorrectly) that a straight line
on earth ([Geodesic](https://en.wikipedia.org/wiki/Geodesic)) is actually contained in a plane and
that this plane contains the center of the spheroid; then it becomes a trivial problem using the
algebraic geometry toolbox.

The `.sml` files issued by [scoremyline](https://scoremyline.com/) contain a lot of sample distances
and deviations for us to compare and we did not find any discrepancies in computation.

What we did find however is a non-documented choice for track points before the target line's start
and after the target line end to be discarded with regards to max. deviation (medal ranking and
score), but apparently not when it comes to the *leniency* setting (what percentage of worst point
should be ignored).

### The *Burdell* score
The basic idea of the *Burdell score* is [as follows](./src/burdell.rs):
  1. divide the target line in segments
  2. evaluate the maximum deviation for each segment
  3. derive a penalty for the such a deviation
  4. aggregate penalties of each segments

Following the description given on [scoremyline](https://scoremyline.com/) we manage to be rather
consistent with sample values, with few puzzling exceptions though:

| Mission            | Pro          | Pro err.     | Amateur      | Amateur err. | Newbie       | Newbie err.  |
|:-------------------|-------------:|-------------:|-------------:|-------------:|-------------:|-------------:|
| archie-iom         |        98.78 |            - |        99.84 |            - |        99.96 |            - |
| archie-scotland    |        90.91 |            - |        99.00 |         0.04 |        99.83 |         0.02 |
| archie-wales-run   |        98.41 |            - |        99.81 |         0.01 |        99.96 |         0.01 |
| archie-wales-walk  |        98.32 |            - |        99.80 |         0.01 |        99.96 |            - |
| geowizard-iom      |        92.56 |            - |        99.14 |         0.04 |        99.83 |         0.01 |
| geowizard-norway   |        97.61 |            - |        99.72 |            - |        99.94 |            - |
| geowizard-scotland |        99.58 |            - |        99.95 |            - |        99.99 |            - |
| geowizard-wales1a  |         0.00 |            - |         0.00 |            - |        44.33 |     **3.70** |
| geowizard-wales1b  |        63.41 |            - |        95.94 |       *0.15* |        99.35 |         0.05 |
| geowizard-wales2   |        59.82 |            - |        95.76 |       *0.17* |        99.16 |         0.01 |
| geowizard-wales3   |        90.39 |            - |        98.87 |            - |        99.82 |         0.01 |
| geowizard-wales4   |        93.41 |            - |        99.30 |         0.02 |        99.89 |         0.01 |
| hiiumaa            |         0.00 |            - |        84.29 |       *0.70* |        97.25 |         0.09 |
| muhu               |        75.00 |         0.01 |        97.10 |         0.07 |        99.47 |         0.02 |
| new-forest         |         0.00 |            - |        65.84 |     **1.00** |        94.75 |       *0.19* |
| saaremaa           |        93.98 |            - |        99.27 |         0.02 |        99.84 |            - |
| schaffhausen       |         0.00 |            - |        76.31 |         0.05 |        96.04 |     **1.00** |

It is unclear to us what could explain the differences. More details from Mr. Burdell would probably
help. The fact that the errors are bigger as the level (Pro, Amateur, Newbie) decreases and as such
length of the segments increases, may indicate that our division logic is different.

## How to use it ?
For now there is a library and a CLI tool. You need to compile them
```
$ cargo build --all --release
```


### The CLI tool
The program takes an input file (CSV or GPX) and optionally the start and end positions and displays
the different statistics about about the track.

```
$ target/release/slm-cli --help
Usage: slm-cli[.exe] [OPTIONS] FILE

Arguments:
  FILE  Input file.

Options:
  -s, --start POINT    Route start point.
  -e, --end POINT      Route end point.
  -f, --format FORMAT  Input file format (default: input file extension).
  -h, --help           Show this message.

Values:
  POINT   Comma separated coordinates (latitude, longitude) as decimal degrees; north and east as
          positive values, south and west as negative values. Ex: '52.606,-1.91787'
  FORMAT  csv: one POINT per line (optional header).
          gpx: first track.
```

```
$ target/release/slm-cli fixtures/archie-iom.csv
Route length:             15.0 km
Max. deviation:           21.7 m
Medal rank:               PLATINUM
Burdell score (PRO):      96.0 %
Burdell score (AMATEUR):  99.5 %
Burdell score (NEWBIE):   99.9 %
```

```
$ target/debug/slm-cli fixtures/schaffhausen.gpx
Route length:             12.7 km
Max. deviation:           56.5 m
Medal rank:               SILVER
Burdell score (PRO):      0.0 %
Burdell score (AMATEUR):  50.7 %
Burdell score (NEWBIE):   92.4 %
```
