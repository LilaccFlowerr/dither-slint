pub mod avatar;
pub mod backgrounds;
pub mod bloom;
pub mod color;
pub mod diffuse;
pub mod dot_map;
pub mod empty;
pub mod export;
pub mod field;
pub mod gauge;
pub mod globe;
pub mod glow;
pub mod lively;
pub mod loader;
pub mod matrix;
pub mod noise;
pub mod number;
pub mod progress;
pub mod render;
pub mod skeleton;
pub mod slider;
pub mod stat;
pub mod style;
pub mod switch;
pub mod tabs;
pub mod trend;
pub mod uptime;
pub mod world;

mod blue_noise;

pub use glam;
pub use matrix::Matrix;
#[doc(hidden)]
pub use slint;

#[macro_export]
macro_rules! install {
    ($component:expr) => {{
        use $crate::lively::{Hover, Idle, Layer, Press};
        use $crate::slint::{ComponentHandle as _, Model as _};
        use $crate::style::{FillEdge, FillVariant};
        use $crate::{Matrix, bloom::Bloom, diffuse::Method, glow::Glow, render};

        fn cells(count: i32) -> u32 {
            count.max(0) as u32
        }

        fn trend_frame(frame: &DitherTrendFrame) -> $crate::trend::Frame {
            $crate::trend::Frame {
                width: frame.width,
                height: frame.height,
                margin: $crate::trend::Margin { left: frame.left, right: frame.right, top: frame.top, bottom: frame.bottom },
                domain: (frame.min != frame.max).then_some((frame.min, frame.max)),
            }
        }

        fn uptime_day(day: &DitherUptimeDay) -> $crate::uptime::Day {
            $crate::uptime::Day {
                date: day.date.to_string(),
                uptime: day.uptime,
                note: day.note.to_string(),
                status: day.custom_status.then(|| $crate::uptime::Status::ALL[day.status as usize]),
            }
        }

        fn uptime_status(status: $crate::uptime::Status) -> DitherUptimeStatus {
            match status {
                $crate::uptime::Status::Up => DitherUptimeStatus::Up,
                $crate::uptime::Status::Degraded => DitherUptimeStatus::Degraded,
                $crate::uptime::Status::Down => DitherUptimeStatus::Down,
                $crate::uptime::Status::None => DitherUptimeStatus::None,
            }
        }

        fn gauge_spec(spec: &DitherGaugeSpec) -> $crate::gauge::Gauge {
            $crate::gauge::Gauge {
                variant: $crate::gauge::Variant::ALL[spec.variant as usize],
                size: spec.size,
                segments: spec.segments.max(0) as u32,
                ratio: spec.ratio,
                cell: spec.cell,
            }
        }

        fn number_roll(roll: &DitherNumberRoll) -> $crate::number::Roll {
            $crate::number::Roll { from: roll.from, to: roll.to, decimals: roll.decimals.max(0) as usize, duration: roll.duration, stagger: roll.stagger }
        }

        fn markers(markers: &$crate::slint::ModelRc<DitherMarker>) -> Vec<$crate::world::Marker> {
            markers
                .iter()
                .map(|m| $crate::world::Marker { at: $crate::world::Place { lat: m.lat, lon: m.lon }, color: (m.color.alpha() > 0).then_some(m.color) })
                .collect()
        }

        fn routes(arcs: &$crate::slint::ModelRc<DitherArc>) -> Vec<$crate::world::Route> {
            arcs.iter()
                .map(|a| $crate::world::Route {
                    from: $crate::world::Place { lat: a.from_lat, lon: a.from_lon },
                    to: $crate::world::Place { lat: a.to_lat, lon: a.to_lon },
                    color: (a.color.alpha() > 0).then_some(a.color),
                })
                .collect()
        }

        fn bounds(bounds: &DitherBounds) -> $crate::dot_map::Bounds {
            $crate::dot_map::Bounds { north: bounds.north, south: bounds.south, west: bounds.west, east: bounds.east }
        }

        let engine = $component.global::<DitherEngine>();

        engine.on_linear_gradient(|columns, rows, angle, from, to, matrix, palette, levels| {
            let matrix = Matrix::ALL[matrix as usize];
            let palette = palette.iter().collect();
            let options = render::GradientOptions { angle, from, to, matrix, palette, levels: cells(levels) };
            render::linear_gradient(cells(columns), cells(rows), &options)
        });
        engine.on_fill(|columns, rows, variant, edge, length, from, to, matrix, color, frame| {
            let options = render::FillOptions {
                variant: FillVariant::ALL[variant as usize],
                edge: FillEdge::ALL[edge as usize],
                length,
                from,
                to,
                matrix: Matrix::ALL[matrix as usize],
                color,
                frame,
            };
            render::fill(cells(columns), cells(rows), &options)
        });
        engine.on_rule(|columns, rows, vertical, fade_start, fade_end, fade, strength, variant, matrix, color| {
            let options = render::RuleOptions {
                vertical,
                fade_start,
                fade_end,
                fade,
                strength,
                variant: FillVariant::ALL[variant as usize],
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::rule(cells(columns), cells(rows), &options)
        });
        engine.on_slider_fill(|columns, rows, end, hot, matrix, color| {
            let options = render::SliderOptions {
                start: 0,
                end,
                both: false,
                hot,
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::slider_fill(cells(columns), cells(rows), &options)
        });
        engine.on_slider_range_fill(|columns, rows, start, end, both, hot, matrix, color| {
            let options = render::SliderOptions {
                start,
                end,
                both,
                hot,
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::slider_fill(cells(columns), cells(rows), &options)
        });
        engine.on_avatar(|name, fill, color| $crate::avatar::avatar(&name, FillVariant::ALL[fill as usize], color));
        engine.on_avatar_hue(|name| $crate::avatar::hue(&name) as i32);
        engine.on_loader(|variant, height, cell, time, matrix, color| {
            $crate::loader::loader($crate::loader::Variant::ALL[variant as usize], height, cell, time, Matrix::ALL[matrix as usize], color)
        });
        engine.on_skeleton(|columns, rows, cell, fill, animation, phase, matrix, color| {
            let options = $crate::skeleton::SkeletonOptions {
                cell,
                fill: FillVariant::ALL[fill as usize],
                animation: $crate::skeleton::Animation::ALL[animation as usize],
                phase,
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            $crate::skeleton::skeleton(cells(columns), cells(rows), &options)
        });
        engine.on_empty(|illustration, backdrop, matrix, color| {
            let layer = if backdrop { $crate::empty::Layer::Backdrop } else { $crate::empty::Layer::Object };
            $crate::empty::empty($crate::empty::Illustration::ALL[illustration as usize], layer, Matrix::ALL[matrix as usize], color)
        });
        engine.on_progress_spans(|parts, max| {
            let values: Vec<f32> = parts.iter().map(|part| part.value).collect();
            let spans: Vec<DitherProgressSpan> =
                $crate::progress::spans(&values, max).into_iter().map(|span| DitherProgressSpan { start: span.start, share: span.share }).collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(spans))
        });
        engine.on_progress_total(|parts| {
            let values: Vec<f32> = parts.iter().map(|part| part.value).collect();
            $crate::progress::total(&values)
        });
        engine.on_trend_area(|values, frame, cell, style| {
            let values: Vec<f32> = values.iter().collect();
            let options = $crate::trend::AreaOptions {
                fill: FillVariant::ALL[style.fill as usize],
                shading: $crate::trend::Shading::ALL[style.shading as usize],
                level: style.level,
                matrix: Matrix::ALL[style.matrix as usize],
                color: style.color,
            };
            $crate::trend::area(&values, &trend_frame(&frame), cell, &options)
        });
        engine.on_trend_bloom(|values, frame, cell, bloom| {
            let values: Vec<f32> = values.iter().collect();
            let options = $crate::trend::BloomOptions { area: bloom.area, bloom: Bloom::ALL[bloom.bloom as usize], color: bloom.color };
            $crate::trend::bloom(&values, &trend_frame(&frame), cell, &options)
        });
        engine.on_bloom_padding(|bloom, cell| $crate::bloom::padding(Bloom::ALL[bloom as usize], cell) as i32);
        engine.on_rect_bloom(|columns, rows, cell, bloom, color| $crate::bloom::rect(cells(columns), cells(rows), cell, Bloom::ALL[bloom as usize], color));
        engine.on_trend_line(|values, frame| {
            let values: Vec<f32> = values.iter().collect();
            $crate::trend::path(&$crate::trend::points(&values, &trend_frame(&frame))).into()
        });
        engine.on_trend_points(|values, frame| {
            let values: Vec<f32> = values.iter().collect();
            let points: Vec<DitherTrendPoint> =
                $crate::trend::points(&values, &trend_frame(&frame)).into_iter().map(|point| DitherTrendPoint { x: point.x, y: point.y }).collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(points))
        });
        engine.on_trend_summary(|label, values| {
            let values: Vec<f32> = values.iter().collect();
            $crate::trend::summary(&label, &values).into()
        });
        engine.on_format_compact(|value| $crate::stat::format_compact(value).into());
        engine.on_format_decimals(|value, decimals| $crate::stat::format_decimals(value, decimals.max(0) as usize).into());
        engine.on_gauge_layout(|spec| {
            let layout = $crate::gauge::layout(&gauge_spec(&spec));
            DitherGaugeLayout { height: layout.height, thickness: layout.thickness, range_width: layout.range_width, reveal: $crate::gauge::reveal(&layout) }
        });
        engine.on_gauge_arc(|spec, style| {
            let style = $crate::gauge::ArcStyle {
                fill: FillVariant::ALL[style.fill as usize],
                matrix: Matrix::ALL[style.matrix as usize],
                color: style.color,
                track: style.track_color,
                elapsed: style.elapsed,
            };
            $crate::gauge::arc(&gauge_spec(&spec), &style)
        });
        engine.on_gauge_bloom(|spec, bloom, color| $crate::gauge::glow(&gauge_spec(&spec), Bloom::ALL[bloom as usize], color));
        engine.on_number_glyphs(|roll, elapsed| {
            let glyphs: Vec<DitherNumberGlyph> = $crate::number::glyphs(&number_roll(&roll), elapsed)
                .into_iter()
                .map(|glyph| match glyph {
                    $crate::number::Glyph::Digit { row, tint } => DitherNumberGlyph { digit: true, text: "".into(), row, tint },
                    $crate::number::Glyph::Static(char) => DitherNumberGlyph { digit: false, text: char.to_string().into(), row: 0.0, tint: 0.0 },
                })
                .collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(glyphs))
        });
        engine.on_number_length(|roll| $crate::number::length(&number_roll(&roll)));
        engine.on_number_edge(|columns, rows, matrix, color| $crate::number::edge(cells(columns), cells(rows), Matrix::ALL[matrix as usize], color));
        engine.on_uptime_day(|day| {
            let day = uptime_day(&day);
            let status = $crate::uptime::status(&day);
            DitherUptimeInfo {
                status: uptime_status(status),
                status_text: status.text().into(),
                date_text: $crate::uptime::date(&day.date).into(),
                percent_text: if day.uptime < 0.0 { "".into() } else { $crate::uptime::percent(day.uptime).into() },
                description: $crate::uptime::describe(&day).into(),
            }
        });
        engine.on_uptime_summary(|days| {
            let days: Vec<_> = days.iter().map(|day| uptime_day(&day)).collect();
            let status = days.last().map_or($crate::uptime::Status::None, $crate::uptime::status);
            DitherUptimeSummary {
                status: uptime_status(status),
                status_text: status.text().into(),
                overall_text: $crate::uptime::overall(&days).map($crate::uptime::percent).unwrap_or_default().into(),
            }
        });
        engine.on_stat_change(|delta, good| {
            use $crate::stat::{Direction, Outcome};
            let good = match good {
                DitherStatDirection::Up => Direction::Up,
                DitherStatDirection::Down => Direction::Down,
                DitherStatDirection::Flat => Direction::Flat,
            };
            let change = $crate::stat::describe_change(delta, good);
            DitherStatChange {
                text: change.text.into(),
                direction: match change.direction {
                    Direction::Up => DitherStatDirection::Up,
                    Direction::Down => DitherStatDirection::Down,
                    Direction::Flat => DitherStatDirection::Flat,
                },
                outcome: match change.outcome {
                    Outcome::Good => DitherStatOutcome::Good,
                    Outcome::Bad => DitherStatOutcome::Bad,
                    Outcome::Neutral => DitherStatOutcome::Neutral,
                },
            }
        });
        engine.on_format_number(|value| $crate::stat::format_number(value).into());
        engine.on_picture(|source, columns, rows, method, threshold, palette| {
            let method = Method::ALL[method as usize];
            let options = render::PictureOptions { method, threshold, palette: palette.iter().collect() };
            render::picture(&source, cells(columns), cells(rows), &options)
        });
        engine.on_card_glow(|columns, rows, glow, reach, strength, boost, rim, radius, fill, matrix, color| {
            let options = $crate::glow::GlowOptions {
                glow: Glow::ALL[glow as usize],
                reach,
                strength,
                boost,
                rim,
                radius,
                fill: FillVariant::ALL[fill as usize],
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            $crate::glow::card_glow(cells(columns), cells(rows), &options)
        });
        engine.on_card_sparkles(|columns, rows, cell, tick, glow, reach, color| {
            let layer = Layer::new(columns, rows, 1.0, color, Matrix::Bayer4);
            $crate::lively::glow_sparkles(&layer, cell, tick, Glow::ALL[glow as usize], reach)
        });
        engine.on_shader(|field, columns, rows, time, params, palette, matrix, levels| {
            let Some(field) = $crate::field::lookup(&field) else {
                return $crate::slint::Image::default();
            };
            let (params, palette): (Vec<f32>, Vec<_>) = (params.iter().collect(), palette.iter().collect());
            $crate::field::render(&$crate::field::Shader {
                field: field.as_ref(),
                columns: cells(columns),
                rows: cells(rows),
                time,
                params: &params,
                palette: &palette,
                matrix: Matrix::ALL[matrix as usize],
                levels: cells(levels),
            })
        });
        engine.on_globe(|columns, rows, view, style, markers_in, arcs_in| {
            let (markers, arcs) = (markers(&markers_in), routes(&arcs_in));
            let globe = $crate::globe::Globe {
                land: style.land,
                accent: style.accent,
                markers: &markers,
                arcs: &arcs,
                light: style.light,
                ocean: style.ocean,
                atmosphere: style.atmosphere,
                matrix: Matrix::ALL[style.matrix as usize],
            };
            $crate::globe::globe(cells(columns), cells(rows), &globe, &$crate::globe::View { spin: view.spin, tilt: view.tilt, time: view.time })
        });
        engine.on_dot_map(|scale, time, animated, style, markers_in, arcs_in| {
            let (markers, arcs) = (markers(&markers_in), routes(&arcs_in));
            let map = $crate::dot_map::DotMap {
                markers: &markers,
                arcs: &arcs,
                color: style.color,
                accent: style.accent,
                columns: cells(style.columns),
                round: style.round,
                halo: style.halo,
                ocean: style.ocean,
                bounds: bounds(&style.bounds),
            };
            $crate::dot_map::dot_map(&map, cells(scale), animated.then_some(time))
        });
        engine.on_dot_map_rows(|columns, area| $crate::dot_map::Grid::new(cells(columns), bounds(&area)).rows as i32);
        engine.on_mix(render::mix);
        engine.on_random_beams(|| {
            let colors: Vec<$crate::slint::Color> =
                $crate::color::random_pair().into_iter().map(|[r, g, b, a]| $crate::slint::Color::from_argb_f32(a, r, g, b)).collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(colors))
        });
        engine.on_hover(|kind, columns, rows, t, x, y, level, strength, color, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::hover(Hover::ALL[kind as usize], &layer, t, (x, y), level)
        });
        engine.on_press(|kind, columns, rows, t, x, y, seed, strength, color, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::press(Press::ALL[kind as usize], &layer, t, (x, y), seed)
        });
        engine.on_idle(|kind, columns, rows, t, strength, color, cell, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::idle(Idle::ALL[kind as usize], &layer, t, cell)
        });
        engine.on_tabs_indicator(|columns, rows, cell, variant, kind, color, fx, fy, fw, fh, tx, ty, tw, th, progress| {
            let variant = match variant {
                DitherTabsVariant::Pill => $crate::tabs::TabsVariant::Pill,
                DitherTabsVariant::Underline => $crate::tabs::TabsVariant::Underline,
            };
            let from = $crate::tabs::BoxRect { x: fx, y: fy, w: fw, h: fh };
            let to = $crate::tabs::BoxRect { x: tx, y: ty, w: tw, h: th };
            $crate::tabs::tabs_indicator(
                cells(columns),
                cells(rows),
                cell,
                variant,
                Matrix::ALL[kind as usize],
                color,
                from,
                to,
                progress,
            )
        });
    }};
}

use std::collections::HashMap;
use std::path::PathBuf;

pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("dither".to_string(), library_path())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diffuse::Method;
    use crate::glow::Glow;
    use crate::lively::{Hover, Idle, Press};
    use crate::style::{FillEdge, FillVariant};

    const ENGINE: &str = include_str!("../ui/engine.slint");

    fn in_slint(name: &str) -> Vec<String> {
        let start = ENGINE.find(&format!("export enum {name} {{")).unwrap();
        let body = &ENGINE[start..];
        let body = &body[body.find('{').unwrap() + 1..body.find('}').unwrap()];
        body.split(',').map(|variant| variant.trim().replace('-', "").to_lowercase()).collect()
    }

    fn in_rust<T: std::fmt::Debug>(all: &[T]) -> Vec<String> {
        all.iter().map(|variant| format!("{variant:?}").to_lowercase()).collect()
    }

    #[test]
    fn rust_enums_follow_the_order_of_the_slint_ones() {
        assert_eq!(in_rust(&Matrix::ALL), in_slint("DitherMatrix"));
        assert_eq!(in_rust(&FillVariant::ALL), in_slint("DitherFill"));
        assert_eq!(in_rust(&FillEdge::ALL), in_slint("DitherEdge"));
        assert_eq!(in_rust(&Glow::ALL), in_slint("DitherGlow"));
        assert_eq!(in_rust(&Hover::ALL), in_slint("DitherHover"));
        assert_eq!(in_rust(&Press::ALL), in_slint("DitherPress"));
        assert_eq!(in_rust(&Idle::ALL), in_slint("DitherIdle"));
        assert_eq!(in_rust(&Method::ALL), in_slint("DitherMethod"));
        assert_eq!(in_rust(&crate::loader::Variant::ALL), in_slint("DitherLoaderVariant"));
        assert_eq!(in_rust(&crate::skeleton::Animation::ALL), in_slint("DitherSkeletonAnimation"));
        assert_eq!(in_rust(&crate::empty::Illustration::ALL), in_slint("DitherIllustration"));
        assert_eq!(in_rust(&crate::stat::Direction::ALL), in_slint("DitherStatDirection"));
        assert_eq!(in_rust(&crate::stat::Outcome::ALL), in_slint("DitherStatOutcome"));
        assert_eq!(in_rust(&crate::trend::Shading::ALL), in_slint("DitherTrendShading"));
        assert_eq!(in_rust(&crate::bloom::Bloom::ALL), in_slint("DitherBloom"));
        assert_eq!(in_rust(&crate::uptime::Status::ALL), in_slint("DitherUptimeStatus"));
        assert_eq!(in_rust(&crate::gauge::Variant::ALL), in_slint("DitherGaugeVariant"));
    }
}
