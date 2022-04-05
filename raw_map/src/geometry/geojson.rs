use anyhow::Result;

use abstutil::Timer;
use geom::{Distance, GPSBounds, PolyLine};

use crate::geometry::InputRoad;
use crate::{OriginalRoad, RawMap};

impl RawMap {
    pub fn save_osm2polygons_input(&self, roads: Vec<OriginalRoad>) -> Result<()> {
        let mut features = Vec::new();
        for id in roads {
            let road = crate::initial::Road::new(self, id)?;
            let mut properties = serde_json::Map::new();
            properties.insert("osm_way_id".to_string(), id.osm_way_id.0.into());
            properties.insert("src_i".to_string(), id.i1.0.into());
            properties.insert("dst_i".to_string(), id.i2.0.into());
            properties.insert(
                "half_width".to_string(),
                road.half_width.inner_meters().into(),
            );
            features.push(geojson::Feature {
                geometry: Some(road.trimmed_center_pts.to_geojson(Some(&self.gps_bounds))),
                properties: Some(properties),
                bbox: None,
                id: None,
                foreign_members: None,
            });
        }
        let fc = geojson::FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        };
        let gj = geojson::GeoJson::from(fc);
        abstio::write_json("osmpolygons_input.json".to_string(), &gj);
        Ok(())
    }
}

pub fn read_geojson_input(path: String, gps: &GPSBounds) -> Result<Vec<InputRoad>> {
    let geojson: geojson::GeoJson = abstio::maybe_read_json(path, &mut Timer::throwaway())?;
    let mut roads = Vec::new();
    if let geojson::GeoJson::FeatureCollection(collection) = geojson {
        for feature in collection.features {
            let center_pts = PolyLine::from_geojson(&feature, Some(gps))?;
            let osm_way_id = feature
                .property("osm_way_id")
                .and_then(|x| x.as_i64())
                .unwrap();
            let src_i = feature.property("src_i").and_then(|x| x.as_i64()).unwrap();
            let dst_i = feature.property("dst_i").and_then(|x| x.as_i64()).unwrap();
            let id = OriginalRoad::new(osm_way_id, (src_i, dst_i));
            let half_width = Distance::meters(
                feature
                    .property("half_width")
                    .and_then(|x| x.as_f64())
                    .unwrap(),
            );
            roads.push(InputRoad {
                id,
                center_pts,
                half_width,
            });
        }
    }
    Ok(roads)
}

// TODO Can we do serde magic?
// (a geo type, a serializable object)
