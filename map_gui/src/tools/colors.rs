use std::collections::HashMap;

use abstutil::Counter;
use geom::{Circle, Distance, Line, Polygon, Pt2D};
use map_model::{BuildingID, IntersectionID, LaneID, Map, ParkingLotID, RoadID, TransitStopID};
use widgetry::mapspace::{ToggleZoomed, ToggleZoomedBuilder};
use widgetry::{Color, EventCtx, Fill, GeomBatch, Line, LinearGradient, Text, Widget};

use crate::AppLike;

pub struct ColorDiscrete<'a> {
    map: &'a Map,
    // pub so callers can add stuff in before building
    pub draw: ToggleZoomedBuilder,
    // Store both, so we can build the legend in the original order later
    pub categories: Vec<(String, Color)>,
    colors: HashMap<String, Color>,
}

impl<'a> ColorDiscrete<'a> {
    pub fn new<I: Into<String>>(
        app: &'a dyn AppLike,
        categories: Vec<(I, Color)>,
    ) -> ColorDiscrete<'a> {
        let mut draw = ToggleZoomed::builder();
        draw.unzoomed.push(
            app.cs().fade_map_dark,
            app.map().get_boundary_polygon().clone(),
        );
        let categories: Vec<(String, Color)> =
            categories.into_iter().map(|(k, v)| (k.into(), v)).collect();
        ColorDiscrete {
            map: app.map(),
            draw,
            colors: categories.iter().cloned().collect(),
            categories,
        }
    }

    pub fn no_fading<I: Into<String>>(
        app: &'a dyn AppLike,
        categories: Vec<(I, Color)>,
    ) -> ColorDiscrete<'a> {
        let mut c = ColorDiscrete::new(app, categories);
        c.draw.unzoomed = GeomBatch::new();
        c
    }

    pub fn add_l<I: AsRef<str>>(&mut self, l: LaneID, category: I) {
        let color = self.colors[category.as_ref()];
        self.draw
            .unzoomed
            .push(color, self.map.get_parent(l).get_thick_polygon());
        let lane = self.map.get_l(l);
        self.draw
            .zoomed
            .push(color.alpha(0.4), lane.get_thick_polygon());
    }

    pub fn add_r<I: AsRef<str>>(&mut self, r: RoadID, category: I) {
        let color = self.colors[category.as_ref()];
        self.draw
            .unzoomed
            .push(color, self.map.get_r(r).get_thick_polygon());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_r(r).get_thick_polygon());
    }

    pub fn add_i<I: AsRef<str>>(&mut self, i: IntersectionID, category: I) {
        let color = self.colors[category.as_ref()];
        self.draw
            .unzoomed
            .push(color, self.map.get_i(i).polygon.clone());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_i(i).polygon.clone());
    }

    pub fn add_b<I: AsRef<str>>(&mut self, b: BuildingID, category: I) {
        let color = self.colors[category.as_ref()];
        self.draw
            .unzoomed
            .push(color, self.map.get_b(b).polygon.clone());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_b(b).polygon.clone());
    }

    pub fn add_ts<I: AsRef<str>>(&mut self, ts: TransitStopID, category: I) {
        let color = self.colors[category.as_ref()];
        let pt = self.map.get_ts(ts).sidewalk_pos.pt(self.map);
        self.draw.zoomed.push(
            color.alpha(0.4),
            Circle::new(pt, Distance::meters(5.0)).to_polygon(),
        );
        self.draw
            .unzoomed
            .push(color, Circle::new(pt, Distance::meters(15.0)).to_polygon());
    }

    pub fn build(self, ctx: &EventCtx) -> (ToggleZoomed, Widget) {
        let legend = self
            .categories
            .into_iter()
            .map(|(name, color)| ColorLegend::row(ctx, color, name))
            .collect();
        (self.draw.build(ctx), Widget::col(legend))
    }
}

pub struct ColorLegend {}

impl ColorLegend {
    pub fn row(ctx: &EventCtx, color: Color, label: impl AsRef<str>) -> Widget {
        let radius = 15.0;
        Widget::row(vec![
            GeomBatch::from(vec![(
                color,
                Circle::new(Pt2D::new(radius, radius), Distance::meters(radius)).to_polygon(),
            )])
            .into_widget(ctx)
            .centered_vert(),
            Text::from(label).wrap_to_pct(ctx, 35).into_widget(ctx),
        ])
    }

    pub fn gradient<I: Into<String>>(
        ctx: &mut EventCtx,
        scale: &ColorScale,
        labels: Vec<I>,
    ) -> Widget {
        assert!(scale.0.len() >= 2);
        let width = 300.0;
        let n = scale.0.len();
        let mut batch = GeomBatch::new();
        let width_each = width / ((n - 1) as f64);
        batch.push(
            Fill::LinearGradient(LinearGradient {
                line: Line::must_new(Pt2D::new(0.0, 0.0), Pt2D::new(width, 0.0)),
                stops: scale
                    .0
                    .iter()
                    .enumerate()
                    .map(|(idx, color)| ((idx as f64) / ((n - 1) as f64), *color))
                    .collect(),
            }),
            Polygon::union_all(
                (0..n - 1)
                    .map(|i| {
                        Polygon::rectangle(width_each, 32.0).translate((i as f64) * width_each, 0.0)
                    })
                    .collect(),
            ),
        );
        // Extra wrapping to make the labels stretch against just the scale, not everything else
        // TODO Long labels aren't nicely lined up with the boundaries between buckets
        Widget::col(vec![
            batch.into_widget(ctx),
            Widget::custom_row(
                labels
                    .into_iter()
                    .map(|lbl| Line(lbl).small().into_widget(ctx))
                    .collect(),
            )
            .evenly_spaced(),
        ])
        .container()
    }

    pub fn categories(ctx: &mut EventCtx, pairs: Vec<(Color, &str)>) -> Widget {
        assert!(pairs.len() >= 2);
        let width = 300.0;
        let n = pairs.len();
        let mut batch = GeomBatch::new();
        let width_each = width / ((n - 1) as f64);
        for (idx, (color, _)) in pairs.iter().enumerate() {
            batch.push(
                *color,
                Polygon::rectangle(width_each, 32.0).translate((idx as f64) * width_each, 0.0),
            );
        }
        // Extra wrapping to make the labels stretch against just the scale, not everything else
        // TODO Long labels aren't nicely lined up with the boundaries between buckets
        Widget::col(vec![
            batch.into_widget(ctx),
            Widget::custom_row(
                pairs
                    .into_iter()
                    .map(|(_, lbl)| Line(lbl).small().into_widget(ctx))
                    .collect(),
            )
            .evenly_spaced(),
        ])
        .container()
    }
}

pub struct DivergingScale {
    low_color: Color,
    mid_color: Color,
    high_color: Color,
    min: f64,
    avg: f64,
    max: f64,
    ignore: Option<(f64, f64)>,
}

impl DivergingScale {
    pub fn new(low_color: Color, mid_color: Color, high_color: Color) -> DivergingScale {
        DivergingScale {
            low_color,
            mid_color,
            high_color,
            min: 0.0,
            avg: 0.5,
            max: 1.0,
            ignore: None,
        }
    }

    pub fn range(mut self, min: f64, max: f64) -> DivergingScale {
        assert!(min < max);
        self.min = min;
        self.avg = (min + max) / 2.0;
        self.max = max;
        self
    }

    pub fn ignore(mut self, from: f64, to: f64) -> DivergingScale {
        assert!(from < to);
        self.ignore = Some((from, to));
        self
    }

    pub fn eval(&self, value: f64) -> Option<Color> {
        let value = value.clamp(self.min, self.max);
        if let Some((from, to)) = self.ignore {
            if value >= from && value <= to {
                return None;
            }
        }
        if value <= self.avg {
            Some(
                self.low_color
                    .lerp(self.mid_color, (value - self.min) / (self.avg - self.min)),
            )
        } else {
            Some(
                self.mid_color
                    .lerp(self.high_color, (value - self.avg) / (self.max - self.avg)),
            )
        }
    }

    pub fn make_legend<I: Into<String>>(self, ctx: &mut EventCtx, labels: Vec<I>) -> Widget {
        ColorLegend::gradient(
            ctx,
            &ColorScale(vec![self.low_color, self.mid_color, self.high_color]),
            labels,
        )
    }
}

// TODO Bad name
pub struct ColorNetwork<'a> {
    map: &'a Map,
    pub draw: ToggleZoomedBuilder,
}

impl<'a> ColorNetwork<'a> {
    pub fn new(app: &'a dyn AppLike) -> ColorNetwork {
        let mut draw = ToggleZoomed::builder();
        draw.unzoomed.push(
            app.cs().fade_map_dark,
            app.map().get_boundary_polygon().clone(),
        );
        ColorNetwork {
            map: app.map(),
            draw,
        }
    }

    pub fn no_fading(app: &'a dyn AppLike) -> ColorNetwork {
        ColorNetwork {
            map: app.map(),
            draw: ToggleZoomed::builder(),
        }
    }

    pub fn add_l(&mut self, l: LaneID, color: Color) {
        self.draw
            .unzoomed
            .push(color, self.map.get_parent(l).get_thick_polygon());
        let lane = self.map.get_l(l);
        self.draw
            .zoomed
            .push(color.alpha(0.4), lane.get_thick_polygon());
    }

    pub fn add_r(&mut self, r: RoadID, color: Color) {
        self.draw
            .unzoomed
            .push(color, self.map.get_r(r).get_thick_polygon());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_r(r).get_thick_polygon());
    }

    pub fn add_i(&mut self, i: IntersectionID, color: Color) {
        self.draw
            .unzoomed
            .push(color, self.map.get_i(i).polygon.clone());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_i(i).polygon.clone());
    }

    pub fn add_b(&mut self, b: BuildingID, color: Color) {
        self.draw
            .unzoomed
            .push(color, self.map.get_b(b).polygon.clone());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_b(b).polygon.clone());
    }

    pub fn add_pl(&mut self, pl: ParkingLotID, color: Color) {
        self.draw
            .unzoomed
            .push(color, self.map.get_pl(pl).polygon.clone());
        self.draw
            .zoomed
            .push(color.alpha(0.4), self.map.get_pl(pl).polygon.clone());
    }

    // Order the roads by count, then interpolate a color based on position in that ordering.
    pub fn ranked_roads(&mut self, counter: Counter<RoadID>, scale: &ColorScale) {
        let roads = counter.sorted_asc();
        let len = roads.len() as f64;
        for (idx, list) in roads.into_iter().enumerate() {
            let color = scale.eval((idx as f64) / len);
            for r in list {
                self.add_r(r, color);
            }
        }
    }
    pub fn ranked_intersections(&mut self, counter: Counter<IntersectionID>, scale: &ColorScale) {
        let intersections = counter.sorted_asc();
        let len = intersections.len() as f64;
        for (idx, list) in intersections.into_iter().enumerate() {
            let color = scale.eval((idx as f64) / len);
            for i in list {
                self.add_i(i, color);
            }
        }
    }

    // Interpolate a color for each road based on the max count.
    pub fn pct_roads(&mut self, counter: Counter<RoadID>, scale: &ColorScale) {
        let max = counter.max() as f64;
        for (r, cnt) in counter.consume() {
            self.add_r(r, scale.eval((cnt as f64) / max));
        }
    }
    // Interpolate a color for each intersection based on the max count.
    pub fn pct_intersections(&mut self, counter: Counter<IntersectionID>, scale: &ColorScale) {
        let max = counter.max() as f64;
        for (i, cnt) in counter.consume() {
            self.add_i(i, scale.eval((cnt as f64) / max));
        }
    }

    pub fn build(self, ctx: &EventCtx) -> ToggleZoomed {
        self.draw.build(ctx)
    }
}

pub struct ColorScale(pub Vec<Color>);

impl ColorScale {
    pub fn eval(&self, pct: f64) -> Color {
        let (low, pct) = self.inner_eval(pct);
        self.0[low].lerp(self.0[low + 1], pct)
    }

    #[allow(unused)]
    pub fn from_colorous(gradient: colorous::Gradient) -> ColorScale {
        let n = 7;
        ColorScale(
            (0..n)
                .map(|i| {
                    let c = gradient.eval_rational(i, n);
                    Color::rgb(c.r as usize, c.g as usize, c.b as usize)
                })
                .collect(),
        )
    }

    fn inner_eval(&self, pct: f64) -> (usize, f64) {
        assert!((0.0..=1.0).contains(&pct));
        // What's the interval between each pair of colors?
        let width = 1.0 / (self.0.len() - 1) as f64;
        let low = (pct / width).floor() as usize;
        if low == self.0.len() - 1 {
            return (low - 1, 1.0);
        }
        (low, (pct % width) / width)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_scale() {
        use super::ColorScale;
        use widgetry::Color;

        let two = ColorScale(vec![Color::BLACK, Color::WHITE]);
        assert_same((0, 0.0), two.inner_eval(0.0));
        assert_same((0, 0.5), two.inner_eval(0.5));
        assert_same((0, 1.0), two.inner_eval(1.0));

        let three = ColorScale(vec![Color::BLACK, Color::RED, Color::WHITE]);
        assert_same((0, 0.0), three.inner_eval(0.0));
        assert_same((0, 0.4), three.inner_eval(0.2));
        assert_same((1, 0.0), three.inner_eval(0.5));
        assert_same((1, 0.4), three.inner_eval(0.7));
        assert_same((1, 1.0), three.inner_eval(1.0));
    }

    fn assert_same(expected: (usize, f64), actual: (usize, f64)) {
        assert_eq!(expected.0, actual.0);
        if (expected.1 - actual.1).abs() > 0.0001 {
            panic!("{:?} != {:?}", expected, actual);
        }
    }
}
