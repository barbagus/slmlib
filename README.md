# slmlib
Straight line missions utilities

## Objective
Provide a collection of open-source utilities for [straight line missions](https://en.wikipedia.org/wiki/GeoWizard#Straight_line_missions)
(attempts to cross a geographic area in a straight line) enthusiasts.

## Rationale
At this date, the *go to* online utility seems to be the [scoremyline.com](https://scoremyline.com/)
website. Given a *target line* and GPS trace, it provides basic stats (target line length, maximum
deviation), GeoWizard's medal ranking (platinum, gold, silver and bronze) as well as a scoring
scheme known to the community as the *Burdell score* (named after the author of this website).
However, the mathematics and algorithms involved in such computations however are:
  1. not particularly well specified
  2. not trivial,
  3. not open source.

The third point implies in particular that details and correctness of implementations cannot be
subjected to scrutiny. But it also makes it more difficult for those *de facto* standards to ever be
open for discussion or community steering.

*Disclaimer*: We have not been in contact with Mr. Burdell for lack of means to do so (email, social
media, etc.). But we would welcome the oportunity.

## Implementation
The first efforts are to replicate the results of [scoremyline](https://scoremyline.com/). The bulk
of the sample data have been retrieved from the website itself, but also, shout out to the person
behind [Straight Line Wiki](https://straightline.wiki/) for providing us with additional traces.

### Distances and deviations
Measuring distances on earth is a [tricky business](https://en.wikipedia.org/wiki/Geographical_distance)
and it all comes down to the level of precision that you aim for. Thankfully it is an old and very
well studied problem. We have [implemented](./src/wsg84.rs) the very popular [Vincenty](https://en.wikipedia.org/wiki/Vincenty%27s_formulae)
*inverse problem* formula (which is based on an [ellipsoidal](https://en.wikipedia.org/wiki/Spheroid)
model) seeded with the [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System#WGS84) datum. The
level of precision has been chosen to be accurate to the centimeter compared to more involved
methods (well beyond the precision of GPS devices).

Deviations are less trivial as no simple formula explicitly gives you this result. The crux is to
find, for a given point of the track, what the closest point of the target line. Then this is just
another distance computation. See our [implementation](./src/stats.rs) for details.

The `.sml` files issued by [scoremyline](https://scoremyline.com/) contain a lot of sample distances
and deviations for us to compare and we did not find any discrepancies in computation.

What we did find however is a non-documented choice for track points before and after the target
line ends to be discarded regards to max. deviation (medal ranking and score), but apparently not
when it comes to the *leniency* setting (what percentage of worst point should be ignored).

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
| archie-scotland    |        90.91 |            - |        99.00 |       *0.04* |        99.83 |       *0.02* |
| archie-wales-run   |        98.41 |            - |        99.81 |       *0.01* |        99.96 |       *0.01* |
| archie-wales-walk  |        98.32 |            - |        99.80 |       *0.01* |        99.96 |            - |
| geowizard-iom      |        92.56 |            - |        99.14 |       *0.04* |        99.83 |       *0.01* |
| geowizard-norway   |        97.61 |            - |        99.72 |            - |        99.94 |            - |
| geowizard-scotland |        99.58 |            - |        99.95 |            - |        99.99 |            - |
| geowizard-wales1a  |         0.00 |            - |         0.00 |            - |        44.33 |   ***3.70*** |
| geowizard-wales1b  |        63.41 |            - |        95.94 |     **0.15** |        99.35 |       *0.05* |
| geowizard-wales2   |        59.82 |            - |        95.76 |     **0.17** |        99.16 |       *0.01* |
| geowizard-wales3   |        90.39 |            - |        98.87 |            - |        99.82 |       *0.01* |
| geowizard-wales4   |        93.41 |            - |        99.30 |       *0.02* |        99.89 |       *0.01* |
| hiiumaa            |         0.00 |            - |        84.29 |     **0.70** |        97.25 |       *0.01* |
| muhu               |        75.00 |       *0.01* |        97.10 |      *0.07*  |        99.47 |       *0.01* |
| new-forest         |         0.00 |            - |        65.84 |   ***1.00*** |        94.75 |       *0.01* |
| saaremaa           |        93.98 |            - |        99.27 |       *0.02* |        99.84 |            - |
| schaffhausen       |         0.00 |            - |        76.31 |       *0.05* |        96.04 |   ***1.00*** |

It is unclear to us what could explain the differences. More details from Mr. Burdell it would
probably help. Or maybe there is just a bug... somewhere :-)

## How to use it ?
For now, this is pretty much just a library with no user interface. A command line tool as well as
a in-browser interface should be the next steps.
