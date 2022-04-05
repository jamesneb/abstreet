//! OSM describes roads as center-lines that intersect. Turn these into road and intersection
//! polygons roughly by
//!
//! 1) treating the road as a PolyLine with a width, so that it has a left and right edge
//! 2) finding the places where the edges of different roads intersect
//! 3) "Trimming back" the center lines to avoid the overlap
//! 4) Producing a polygon for the intersection itsef
//!
//! I wrote a novella about this: <https://a-b-street.github.io/docs/tech/map/geometry/index.html>

mod geojson;
mod implementation;

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use abstutil::Tags;
use geom::{Distance, GPSBounds, PolyLine, Polygon};

use crate::initial::Road;
use crate::{osm, OriginalRoad};

pub use implementation::intersection_polygon;

pub struct InputRoad {
    pub id: OriginalRoad,
    // The true center of the road, including sidewalks. The input is untrimmed when called on the
    // first endpoint, then trimmed on that one side when called on th second endpoint.
    pub center_pts: PolyLine,
    pub half_width: Distance,
}

pub struct Results {
    pub intersection_polygon: Polygon,
    pub trimmed_center_pts: Vec<(OriginalRoad, PolyLine)>,
    pub debug: Vec<(String, Polygon)>,
}

// TODO This may not be the right interface at all to expose / test
pub fn intersection_polygon_v2(
    intersection_id: osm::NodeID,
    input_roads: Vec<InputRoad>,
) -> Result<Results> {
    let mut intersection_roads = BTreeSet::new();
    let mut roads = BTreeMap::new();
    for road in input_roads {
        intersection_roads.insert(road.id);
        roads.insert(
            road.id,
            Road {
                id: road.id,
                src_i: road.id.i1,
                dst_i: road.id.i2,
                trimmed_center_pts: road.center_pts,
                half_width: road.half_width,
                // Unused
                lane_specs_ltr: Vec::new(),
                // TODO Used to decide about on_off_ramp
                osm_tags: Tags::empty(),
            },
        );
    }

    let (intersection_polygon, debug) = intersection_polygon(
        intersection_id,
        intersection_roads,
        &mut roads,
        // No trim_roads_for_merging
        &BTreeMap::new(),
    )?;

    let trimmed_center_pts = roads
        .into_values()
        .map(|road| (road.id, road.trimmed_center_pts))
        .collect();
    let result = Results {
        intersection_polygon,
        trimmed_center_pts,
        debug,
    };
    Ok(result)
}

// TODO Name's bad
// TODO Where's gps come from?
pub fn roundtrip_geojson(input_path: String, gps: &GPSBounds) -> Result<()> {
    let input_roads = geojson::read_geojson_input(input_path, gps)?;
    Ok(())
}
