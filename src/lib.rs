pub mod avatar;
pub mod backgrounds;
pub mod bloom;
pub mod chart;
pub mod color;
pub mod diffuse;
pub mod donut;
pub mod dot_map;
pub mod empty;
pub mod export;
pub mod field;
pub mod gauge;
pub mod globe;
pub mod heatmap;
pub mod glow;
pub mod lively;
pub mod loader;
pub mod logo;
pub mod matrix;
pub mod noise;
pub mod pixel_area;
pub mod number;
pub mod progress;
pub mod radar;
pub mod render;
pub mod reveal;
pub mod skeleton;
pub mod slider;
pub mod stat;
pub mod treemap;
pub mod style;
pub mod switch;
pub mod tabs;
pub mod text;
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

        fn donut_slices(slices: &$crate::slint::ModelRc<DitherDonutSlice>) -> Vec<$crate::donut::Slice> {
            slices.iter().map(|slice| $crate::donut::Slice { value: slice.value, color: slice.color, fill: FillVariant::ALL[slice.fill as usize] }).collect()
        }

        fn donut_spec(spec: &DitherDonutSpec) -> $crate::donut::Spec {
            $crate::donut::Spec {
                size: spec.size,
                cell: spec.cell,
                mode: match spec.mode {
                    DitherDonutMode::Donut => $crate::donut::Mode::Donut,
                    DitherDonutMode::Pie => $crate::donut::Mode::Pie,
                },
                thickness: spec.thickness,
                gap: spec.gap,
                matrix: Matrix::ALL[spec.matrix as usize],
            }
        }

        fn heatmap_input(
            spec: &DitherHeatmapSpec,
            entries: &$crate::slint::ModelRc<DitherHeatmapEntry>,
            grid: &$crate::slint::ModelRc<DitherHeatmapRow>,
            rows: &$crate::slint::ModelRc<$crate::slint::SharedString>,
            columns: &$crate::slint::ModelRc<$crate::slint::SharedString>,
        ) -> $crate::heatmap::Input {
            $crate::heatmap::Input {
                mode: match spec.mode {
                    DitherHeatmapMode::Calendar => $crate::heatmap::Mode::Calendar,
                    DitherHeatmapMode::Matrix => $crate::heatmap::Mode::Matrix,
                },
                entries: entries.iter().map(|entry| (entry.date.to_string(), entry.value)).collect(),
                grid: grid.iter().map(|row| row.values.iter().collect()).collect(),
                rows: rows.iter().map(|row| row.to_string()).collect(),
                columns: columns.iter().map(|column| column.to_string()).collect(),
                weeks: spec.weeks.max(1) as usize,
                week_start: spec.week_start.clamp(0, 1) as usize,
                steps: spec.steps.max(0) as usize,
                max: (spec.max > 0.0).then_some(spec.max),
            }
        }

        fn heatmap_fitted(spec: &DitherHeatmapSpec, input: &$crate::heatmap::Input) -> ($crate::heatmap::Model, $crate::heatmap::Fit) {
            let full = $crate::heatmap::model(input);
            let widest = if input.mode == $crate::heatmap::Mode::Matrix { full.column_labels.iter().map(|(_, text)| text.chars().count()).max().unwrap_or(0) } else { 0 };
            let fit = $crate::heatmap::fit(spec.width, spec.cell, &full, widest);
            let shown = $crate::heatmap::shifted(&full, full.columns - fit.columns);
            (shown, fit)
        }

        fn radar_series(series: &$crate::slint::ModelRc<DitherRadarSeries>, colors: Option<&$crate::slint::ModelRc<$crate::slint::Color>>) -> Vec<$crate::radar::Series> {
            series
                .iter()
                .enumerate()
                .map(|(index, line)| $crate::radar::Series {
                    values: line.values.iter().collect(),
                    color: colors.and_then(|colors| colors.row_data(index)).unwrap_or(line.color),
                    fill: FillVariant::ALL[line.fill as usize],
                })
                .collect()
        }

        fn radar_spec(spec: &DitherRadarSpec) -> $crate::radar::Spec {
            $crate::radar::Spec {
                side: spec.side,
                cell: spec.cell,
                max: (spec.max > 0.0).then_some(spec.max),
                rings: spec.rings.max(1) as usize,
                grid: match spec.grid {
                    DitherRadarGrid::Polygon => $crate::radar::Grid::Polygon,
                    DitherRadarGrid::Circle => $crate::radar::Grid::Circle,
                },
                matrix: Matrix::ALL[spec.matrix as usize],
            }
        }

        fn dissolve_spec(spec: &DitherDissolveSpec) -> $crate::text::Dissolve {
            $crate::text::Dissolve {
                duration: spec.duration,
                stagger: spec.stagger,
                delay: spec.delay,
                matrix: Matrix::ALL[spec.matrix as usize],
                fill: match spec.fill {
                    DitherDissolveFill::Solid => $crate::text::DissolveFill::Solid,
                    DitherDissolveFill::Gradient => $crate::text::DissolveFill::Gradient,
                    DitherDissolveFill::Dotted => $crate::text::DissolveFill::Dotted,
                    DitherDissolveFill::Hatched => $crate::text::DissolveFill::Hatched,
                },
                level: spec.level,
                showing: spec.showing,
            }
        }

        fn scramble_spec(spec: &DitherScrambleSpec) -> $crate::text::Scramble {
            $crate::text::Scramble {
                mode: match spec.mode {
                    DitherScrambleMode::Dither => $crate::text::ScrambleMode::Dither,
                    DitherScrambleMode::Blocks => $crate::text::ScrambleMode::Blocks,
                    DitherScrambleMode::Glyphs => $crate::text::ScrambleMode::Glyphs,
                },
                duration: spec.duration,
                stagger: spec.stagger,
                delay: spec.delay,
                rate: spec.rate,
            }
        }

        fn chart_of(categories: &$crate::slint::ModelRc<$crate::slint::SharedString>, spec: &DitherChartSpec) -> $crate::chart::Chart {
            $crate::chart::Chart {
                band: spec.band,
                stack: match spec.stack {
                    DitherChartStack::None => $crate::chart::Stack::None,
                    DitherChartStack::Stacked => $crate::chart::Stack::Stacked,
                    DitherChartStack::Percent => $crate::chart::Stack::Percent,
                },
                count: categories.row_count(),
                width: spec.width,
                height: spec.height,
                axes: spec.axes,
                zero: spec.zero,
                smooth: spec.smooth,
            }
        }

        fn chart_series(series: &$crate::slint::ModelRc<DitherChartSeries>) -> Vec<$crate::chart::Series> {
            series
                .iter()
                .map(|line| $crate::chart::Series { values: line.values.iter().collect(), low: line.low.iter().collect(), high: line.high.iter().collect(), dashed: line.dashed })
                .collect()
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
        engine.on_reveal(|source, columns, rows, method, threshold, palette, coarsest, progress, end| {
            let method = Method::ALL[method as usize];
            let options = render::PictureOptions { method, threshold, palette: palette.iter().collect() };
            let end = match end {
                DitherRevealEnd::Photo => $crate::reveal::RevealEnd::Photo,
                DitherRevealEnd::Dither => $crate::reveal::RevealEnd::Dither,
            };
            let reveal = $crate::reveal::Reveal { coarsest: cells(coarsest), progress, end };
            $crate::reveal::reveal(&source, cells(columns), cells(rows), &options, &reveal)
        });
        engine.on_logo(|source, columns, rows, logo_source, tone, effect, fill, outline, floor, matrix, color, sweep| {
            let options = $crate::logo::LogoOptions {
                source: match logo_source {
                    DitherLogoSource::Alpha => $crate::logo::LogoSource::Alpha,
                    DitherLogoSource::Dark => $crate::logo::LogoSource::Dark,
                },
                tone: match tone {
                    DitherLogoTone::Mono => $crate::logo::LogoTone::Mono,
                    DitherLogoTone::Original => $crate::logo::LogoTone::Original,
                },
                fill: FillVariant::ALL[fill as usize],
                outline,
                floor,
                matrix: Matrix::ALL[matrix as usize],
                color,
                effect: match effect {
                    DitherLogoEffect::None => $crate::logo::LogoEffect::None,
                    DitherLogoEffect::Shimmer => $crate::logo::LogoEffect::Shimmer,
                    DitherLogoEffect::Scan => $crate::logo::LogoEffect::Scan,
                },
                sweep,
            };
            $crate::logo::logo(&source, cells(columns), cells(rows), &options)
        });
        engine.on_logo_row(|logos, height, gap| {
            $crate::logo::row_width(logos.iter().map(|item| (item.source.size().width, item.source.size().height)), height, gap)
        });
        engine.on_logo_width(|source, height| $crate::logo::logo_width((source.size().width, source.size().height), height));
        engine.on_treemap_layout(|items, capacity, progress| {
            let values: Vec<f32> = items.iter().map(|item| item.value).collect();
            let remainder = items.iter().position(|item| item.remainder);
            let capacity = if capacity > 0.0 { capacity } else { values.iter().map(|value| value.max(0.0)).sum() };
            let layout = $crate::treemap::layout(&values, remainder, capacity, progress);
            let tiles: Vec<DitherTreemapTile> = layout
                .tiles
                .into_iter()
                .map(|tile| DitherTreemapTile { x: tile.rect.x, y: tile.rect.y, w: tile.rect.w, h: tile.rect.h, value: tile.value })
                .collect();
            DitherTreemapLayout { tiles: $crate::slint::ModelRc::new($crate::slint::VecModel::from(tiles)), used: layout.used, capacity }
        });
        engine.on_treemap_format(|value| $crate::treemap::format_value(value).into());
        engine.on_chart_layout(|categories, series, spec| {
            let (chart, series) = (chart_of(&categories, &spec), chart_series(&series));
            let edges = $crate::chart::edges(&series, chart.stack);
            let geometry = $crate::chart::geometry(&chart, &$crate::chart::scale_values(&series, &edges, chart.stack), &spec.unit);
            let ticks: Vec<DitherChartTick> = geometry
                .scale
                .ticks
                .iter()
                .zip(&geometry.tick_labels)
                .map(|(&tick, label)| DitherChartTick { y: $crate::chart::y_at(&geometry.plot, &geometry.scale, tick), label: label.as_str().into() })
                .collect();
            let labels: Vec<DitherChartLabel> = $crate::chart::labels(&geometry.plot, chart.count)
                .into_iter()
                .map(|label| DitherChartLabel { x: label.x, text: categories.row_data(label.index).unwrap_or_default(), align: label.align })
                .collect();
            let columns = chart.band.then(|| $crate::chart::bars(&chart, &geometry, &series));
            let lines: Vec<DitherChartLine> = series
                .iter()
                .zip(&edges)
                .enumerate()
                .map(|(position, (line, edges))| {
                    let tops: Vec<f32> = edges.iter().map(|edge| edge.high).collect();
                    let points = match &columns {
                        Some(columns) => $crate::chart::bar_tops(&chart, &geometry, columns, position),
                        None => $crate::chart::points(&geometry.plot, &geometry.scale, &tops, chart.count),
                    };
                    let curve = $crate::chart::segments(&points, chart.smooth);
                    let path = if columns.is_some() { String::new() } else if line.dashed { $crate::chart::dashed_path(&curve) } else { $crate::chart::path(&points, &curve) };
                    let points: Vec<DitherChartPoint> = points.iter().map(|point| DitherChartPoint { x: point.x, y: point.y }).collect();
                    DitherChartLine { path: path.into(), points: $crate::slint::ModelRc::new($crate::slint::VecModel::from(points)) }
                })
                .collect();
            let tops: Vec<f32> = (0..chart.count)
                .map(|index| edges.iter().filter_map(|row| row.get(index)).map(|edge| $crate::chart::y_at(&geometry.plot, &geometry.scale, edge.high)).fold(f32::INFINITY, f32::min))
                .collect();
            let totals_of = $crate::chart::totals(&chart, &series);
            let totals = totals_of.clone();
            DitherChartLayout {
                left: geometry.plot.left,
                right: geometry.plot.right,
                top: geometry.plot.top,
                bottom: geometry.plot.bottom,
                ticks: $crate::slint::ModelRc::new($crate::slint::VecModel::from(ticks)),
                labels: $crate::slint::ModelRc::new($crate::slint::VecModel::from(labels)),
                lines: $crate::slint::ModelRc::new($crate::slint::VecModel::from(lines)),
                tops: $crate::slint::ModelRc::new($crate::slint::VecModel::from(tops)),
                totals: $crate::slint::ModelRc::new($crate::slint::VecModel::from(totals)),
                peak: $crate::chart::max_index(&totals_of) as i32,
            }
        });
        engine.on_chart_band(|categories, series, spec, index, cell, level, matrix, color| {
            let (chart, series) = (chart_of(&categories, &spec), chart_series(&series));
            let edges = $crate::chart::edges(&series, chart.stack);
            let geometry = $crate::chart::geometry(&chart, &$crate::chart::scale_values(&series, &edges, chart.stack), &spec.unit);
            let Some(line) = series.get(index as usize) else { return $crate::slint::Image::default() };
            let options = $crate::chart::Band { chart: &chart, series: line, geometry: &geometry, cell, level, matrix: Matrix::ALL[matrix as usize], color };
            $crate::chart::band(&options)
        });
        engine.on_chart_area(|categories, series, spec, colors, fill| {
            let (chart, series) = (chart_of(&categories, &spec), chart_series(&series));
            let edges = $crate::chart::edges(&series, chart.stack);
            let geometry = $crate::chart::geometry(&chart, &$crate::chart::scale_values(&series, &edges, chart.stack), &spec.unit);
            let colors: Vec<$crate::slint::Color> = colors.iter().collect();
            let options = $crate::chart::Area {
                chart: &chart,
                series: &series,
                geometry: &geometry,
                colors: &colors,
                cell: fill.cell,
                level: fill.level,
                matrix: Matrix::ALL[fill.matrix as usize],
                fill: FillVariant::ALL[fill.fill as usize],
            };
            $crate::chart::area(&options)
        });
        engine.on_chart_bars(|categories, series, spec, colors, fill, solid| {
            let (chart, series) = (chart_of(&categories, &spec), chart_series(&series));
            let edges = $crate::chart::edges(&series, chart.stack);
            let geometry = $crate::chart::geometry(&chart, &$crate::chart::scale_values(&series, &edges, chart.stack), &spec.unit);
            let columns = $crate::chart::bars(&chart, &geometry, &series);
            let colors: Vec<$crate::slint::Color> = colors.iter().collect();
            let options = $crate::chart::BarFill {
                chart: &chart,
                geometry: &geometry,
                bars: &columns,
                colors: &colors,
                cell: fill.cell,
                level: fill.level,
                matrix: Matrix::ALL[fill.matrix as usize],
                fill: FillVariant::ALL[fill.fill as usize],
                solid: usize::try_from(solid).ok(),
            };
            $crate::chart::bars_image(&options)
        });
        engine.on_chart_readout(|categories, series, spec, index| {
            let (chart, series) = (chart_of(&categories, &spec), chart_series(&series));
            let edges = $crate::chart::edges(&series, chart.stack);
            let rows: Vec<$crate::slint::SharedString> = $crate::chart::readout(&series, &edges, index as usize, &spec.unit, chart.stack).into_iter().map(Into::into).collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(rows))
        });
        engine.on_chart_format(|value, unit| $crate::chart::format_value(value, &unit).into());
        engine.on_donut_layout(|slices, spec| {
            let layout = $crate::donut::layout(&donut_spec(&spec), &donut_slices(&slices));
            let shares: Vec<f32> = layout.arcs.iter().map(|arc| arc.share).collect();
            let middles: Vec<f32> = layout.arcs.iter().map(|arc| (arc.from + arc.to) / 2.0).collect();
            DitherDonutLayout {
                side: layout.side as i32,
                outer: layout.outer,
                inner: layout.inner,
                total: layout.total,
                shares: $crate::slint::ModelRc::new($crate::slint::VecModel::from(shares)),
                middles: $crate::slint::ModelRc::new($crate::slint::VecModel::from(middles)),
            }
        });
        engine.on_donut_image(|slices, colors, spec, sweep, focus| {
            let mut drawn = donut_slices(&slices);
            for (slice, color) in drawn.iter_mut().zip(colors.iter()) {
                slice.color = color;
            }
            $crate::donut::image(&donut_spec(&spec), &drawn, sweep, usize::try_from(focus).ok())
        });
        engine.on_donut_hit(|slices, spec, x, y| $crate::donut::hit(&donut_spec(&spec), &donut_slices(&slices), x, y).map_or(-1, |index| index as i32));
        engine.on_donut_value(|value, unit| $crate::donut::format_value(value, &unit).into());
        engine.on_donut_share(|share| $crate::donut::format_share(share).into());
        engine.on_heatmap_layout(|spec, entries, grid, rows, columns| {
            let input = heatmap_input(&spec, &entries, &grid, &rows, &columns);
            let (model, fit) = heatmap_fitted(&spec, &input);
            let label = |(position, text): &(usize, String)| DitherHeatmapLabel { position: *position as i32, text: text.as_str().into() };
            let column_labels: Vec<DitherHeatmapLabel> = model.column_labels.iter().map(label).collect();
            let row_labels: Vec<DitherHeatmapLabel> = model.row_labels.iter().map(label).collect();
            DitherHeatmapLayout {
                cell: fit.cell,
                columns: model.columns as i32,
                rows: model.rows as i32,
                left: fit.left,
                label_every: fit.label_every as i32,
                peak: model.peak,
                column_labels: $crate::slint::ModelRc::new($crate::slint::VecModel::from(column_labels)),
                row_labels: $crate::slint::ModelRc::new($crate::slint::VecModel::from(row_labels)),
            }
        });
        engine.on_heatmap_image(|spec, entries, grid, rows, columns, color, foreground, focus, reveal| {
            let input = heatmap_input(&spec, &entries, &grid, &rows, &columns);
            let (model, _) = heatmap_fitted(&spec, &input);
            let options = $crate::heatmap::Paint { steps: $crate::heatmap::steps(input.steps), color, foreground, focus: usize::try_from(focus).ok(), reveal };
            $crate::heatmap::image(&model, &options)
        });
        engine.on_heatmap_readout(|spec, entries, grid, rows, columns, column, row| {
            let input = heatmap_input(&spec, &entries, &grid, &rows, &columns);
            let (model, _) = heatmap_fitted(&spec, &input);
            $crate::heatmap::square_at(&model, column, row).map_or_else(Default::default, |square| $crate::heatmap::readout(square, &spec.unit, &spec.units).into())
        });
        engine.on_heatmap_swatch(|bucket, steps, color, foreground| $crate::heatmap::swatch(bucket.max(0) as usize, $crate::heatmap::steps(steps.max(0) as usize), color, foreground));
        engine.on_pixel_area_image(|values, spec, color| {
            let values: Vec<f32> = values.iter().collect();
            let area = $crate::pixel_area::Area {
                columns: spec.columns.max(1) as usize,
                rows: spec.rows.max(1) as usize,
                min: spec.min,
                max: (spec.max != 0.0).then_some(spec.max),
                fill: FillVariant::ALL[spec.fill as usize],
                color,
                reveal: spec.reveal,
                intensity: spec.intensity,
                marker: usize::try_from(spec.marker).ok(),
                stars: spec.stars,
                winking: spec.winking,
                tick: spec.tick,
            };
            $crate::pixel_area::image(&values, &area)
        });
        engine.on_pixel_area_side(|pixels, cell| $crate::pixel_area::side(pixels, cell) as i32);
        engine.on_pixel_area_readout(|label, value| $crate::pixel_area::readout(&label, value).into());
        engine.on_radar_layout(|axes, series, spec| {
            let labels: Vec<String> = axes.iter().map(|axis| axis.to_string()).collect();
            let layout = $crate::radar::layout(&labels, &radar_series(&series, None), &radar_spec(&spec));
            let point = |vertex: &$crate::radar::Vec2| DitherRadarPoint { x: vertex.x, y: vertex.y };
            let rings: Vec<$crate::slint::SharedString> = layout.rings.iter().map(|ring| ring.as_str().into()).collect();
            let spokes: Vec<DitherRadarPoint> = layout.spokes.iter().map(point).collect();
            let axis_labels: Vec<DitherRadarLabel> = layout
                .labels
                .iter()
                .zip(&labels)
                .map(|(label, text)| DitherRadarLabel { x: label.at.x, y: label.at.y, text: text.as_str().into(), align: label.align, line: label.line })
                .collect();
            let shapes: Vec<DitherRadarShape> = layout
                .vertices
                .iter()
                .map(|vertices| DitherRadarShape { points: $crate::slint::ModelRc::new($crate::slint::VecModel::from(vertices.iter().map(point).collect::<Vec<_>>())) })
                .collect();
            DitherRadarLayout {
                radius: layout.radius,
                top: layout.top,
                rings: $crate::slint::ModelRc::new($crate::slint::VecModel::from(rings)),
                spokes: $crate::slint::ModelRc::new($crate::slint::VecModel::from(spokes)),
                labels: $crate::slint::ModelRc::new($crate::slint::VecModel::from(axis_labels)),
                shapes: $crate::slint::ModelRc::new($crate::slint::VecModel::from(shapes)),
            }
        });
        engine.on_radar_image(|axes, series, colors, spec, scale, focus| $crate::radar::image(axes.row_count(), &radar_series(&series, Some(&colors)), &radar_spec(&spec), scale, usize::try_from(focus).ok()));
        engine.on_radar_axis(|axes, x, y, side| $crate::radar::axis_at(axes.max(1) as usize, $crate::radar::Vec2 { x, y }, side).map_or(-1, |axis| axis as i32));
        engine.on_text_cell(|font_size| $crate::text::cell_for(font_size));
        engine.on_text_ink(|columns, rows, ink, time, backdrop| {
            let ink = $crate::text::Ink {
                fill: match ink.fill {
                    DitherTextFill::Gradient => $crate::text::InkFill::Gradient,
                    DitherTextFill::Noise => $crate::text::InkFill::Noise,
                    DitherTextFill::Dotted => $crate::text::InkFill::Dotted,
                    DitherTextFill::Hatched => $crate::text::InkFill::Hatched,
                    DitherTextFill::Solid => $crate::text::InkFill::Solid,
                },
                direction: ink.direction,
                scale: ink.scale,
                from: ink.from,
                to: ink.to,
                ghost: (ink.ghost >= 0.0).then_some(ink.ghost),
                matrix: Matrix::ALL[ink.matrix as usize],
            };
            $crate::text::ink_overlay(cells(columns), cells(rows), &ink, time, backdrop)
        });
        engine.on_dissolve_units(|text, word| {
            let units: Vec<DitherDissolveUnit> = $crate::text::units(&text, word).into_iter().map(|(text, index)| DitherDissolveUnit { text: text.into(), index: index.map_or(-1, |index| index as i32) }).collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(units))
        });
        engine.on_dissolve_ranks(|count, order| {
            let order = match order {
                DitherDissolveOrder::Start => $crate::text::DissolveOrder::Start,
                DitherDissolveOrder::End => $crate::text::DissolveOrder::End,
                DitherDissolveOrder::Center => $crate::text::DissolveOrder::Center,
                DitherDissolveOrder::Edges => $crate::text::DissolveOrder::Edges,
                DitherDissolveOrder::Random => $crate::text::DissolveOrder::Random,
            };
            $crate::slint::ModelRc::new($crate::slint::VecModel::from($crate::text::ranks(count.max(0) as usize, order)))
        });
        engine.on_dissolve_step(|elapsed, rank, spec| $crate::text::dissolve_step(elapsed, rank, &dissolve_spec(&spec)) as i32);
        engine.on_dissolve_overlay(|columns, rows, step, spec, backdrop| $crate::text::dissolve_overlay(cells(columns), cells(rows), step.max(0) as usize, &dissolve_spec(&spec), backdrop));
        engine.on_dissolve_steps(|matrix| $crate::text::dissolve_steps(Matrix::ALL[matrix as usize]) as i32);
        engine.on_scramble_slots(|text, from, keep_same, elapsed, spec, characters| {
            let pool = $crate::text::glyph_pool(&text, &characters);
            let slots: Vec<DitherScrambleSlot> = $crate::text::slots(&text, &from, keep_same, elapsed, &scramble_spec(&spec), &pool)
                .into_iter()
                .map(|slot| DitherScrambleSlot { character: slot.character.into(), phase: slot.phase as i32, level: slot.level, glyph: slot.glyph.into() })
                .collect();
            $crate::slint::ModelRc::new($crate::slint::VecModel::from(slots))
        });
        engine.on_scramble_block(|columns, rows, level, color| $crate::text::block_overlay(cells(columns), cells(rows), level, color));
        engine.on_scramble_settled(|text, from, keep_same, elapsed, spec| $crate::text::settled(&text, &from, keep_same, elapsed, &scramble_spec(&spec)));
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
            $crate::tabs::tabs_indicator(&$crate::tabs::Indicator {
                columns: cells(columns),
                rows: cells(rows),
                cell,
                variant,
                matrix: Matrix::ALL[kind as usize],
                color,
                from,
                to,
                progress,
            })
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
