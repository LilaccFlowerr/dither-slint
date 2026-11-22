use dither_slint::Matrix;
use dither_slint::bloom::Bloom;
use dither_slint::render::{self, FillOptions, GradientOptions, linear_gradient};
use dither_slint::diffuse::Method;
use dither_slint::dot_map::{self, Bounds, DotMap};
use dither_slint::field::{self, Shader};
use dither_slint::globe::{self, Globe, View};
use dither_slint::glow::{self, Glow, GlowOptions};
use dither_slint::lively::{self, Hover, Idle, Layer, Press};
use dither_slint::style::{FillEdge, FillVariant};
use dither_slint::world::{Marker, Place, Route};
use dither_slint::slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};
use wasm_bindgen::prelude::*;

fn colors(rgba: &[u8]) -> Vec<Color> {
    rgba.chunks_exact(4).map(|c| Color::from_argb_u8(c[3], c[0], c[1], c[2])).collect()
}

#[wasm_bindgen]
pub fn gradient(columns: u32, rows: u32, angle: f32, from: f32, to: f32, matrix: usize, palette: &[u8], levels: u32) -> Vec<u8> {
    let options = GradientOptions { angle, from, to, matrix: Matrix::ALL[matrix], palette: colors(palette), levels };
    bytes(linear_gradient(columns, rows, &options))
}

#[wasm_bindgen]
pub fn fill(
    columns: u32,
    rows: u32,
    variant: usize,
    edge: usize,
    length: f32,
    from: f32,
    to: f32,
    color: &[u8],
    frame: bool,
) -> Vec<u8> {
    let color = colors(color)[0];
    let options = FillOptions {
        variant: FillVariant::ALL[variant],
        edge: FillEdge::ALL[edge],
        length,
        from,
        to,
        matrix: Matrix::Bayer4,
        color,
        frame,
    };
    bytes(render::fill(columns, rows, &options))
}

#[wasm_bindgen]
pub fn mix(base: &[u8], other: &[u8], amount: f32) -> Vec<u8> {
    let (base, other) = (colors(base)[0], colors(other)[0]);
    let mixed = render::mix(base, other, amount);
    vec![mixed.red(), mixed.green(), mixed.blue(), mixed.alpha()]
}

/// Latitude, longitude and an RGBA colour for each marker; a transparent colour takes the accent.
fn markers(flat: &[f32]) -> Vec<Marker> {
    flat.chunks_exact(6).map(|m| Marker { at: Place { lat: m[0], lon: m[1] }, color: own_color(&m[2..]) }).collect()
}

/// The latitude and longitude of both ends and an RGBA colour for each arc.
fn routes(flat: &[f32]) -> Vec<Route> {
    flat.chunks_exact(8)
        .map(|a| Route { from: Place { lat: a[0], lon: a[1] }, to: Place { lat: a[2], lon: a[3] }, color: own_color(&a[4..]) })
        .collect()
}

fn own_color(rgba: &[f32]) -> Option<Color> {
    (rgba[3] > 0.0).then(|| Color::from_argb_u8(rgba[3] as u8, rgba[0] as u8, rgba[1] as u8, rgba[2] as u8))
}

fn map_bounds(bounds: &[f32]) -> Bounds {
    Bounds { north: bounds[0], south: bounds[1], west: bounds[2], east: bounds[3] }
}

#[wasm_bindgen]
pub fn globe(
    columns: u32,
    rows: u32,
    spin: f32,
    tilt: f32,
    time: f32,
    land: &[u8],
    accent: &[u8],
    light: f32,
    ocean: f32,
    atmosphere: bool,
    matrix: usize,
    markers_in: &[f32],
    arcs_in: &[f32],
) -> Vec<u8> {
    let (markers, arcs) = (markers(markers_in), routes(arcs_in));
    let style = Globe { land: colors(land)[0], accent: colors(accent)[0], markers: &markers, arcs: &arcs, light, ocean, atmosphere, matrix: Matrix::ALL[matrix] };
    bytes(globe::globe(columns, rows, &style, &View { spin, tilt, time }))
}

#[wasm_bindgen]
pub fn dot_map(
    scale: u32,
    time: f32,
    animated: bool,
    color: &[u8],
    accent: &[u8],
    columns: u32,
    round: bool,
    halo: f32,
    ocean: f32,
    bounds: &[f32],
    markers_in: &[f32],
    arcs_in: &[f32],
) -> Vec<u8> {
    let (markers, arcs) = (markers(markers_in), routes(arcs_in));
    let map = DotMap { markers: &markers, arcs: &arcs, color: colors(color)[0], accent: colors(accent)[0], columns, round, halo, ocean, bounds: map_bounds(bounds) };
    bytes(dot_map::dot_map(&map, scale, animated.then_some(time)))
}

/// The dots across and down of a map.
#[wasm_bindgen]
pub fn dot_map_grid(columns: u32, bounds: &[f32]) -> Vec<u32> {
    let grid = dot_map::Grid::new(columns, map_bounds(bounds));
    vec![grid.columns, grid.rows]
}

fn bytes(image: Image) -> Vec<u8> {
    image.to_rgba8().map(|buffer| buffer.as_bytes().to_vec()).unwrap_or_default()
}

fn layer(columns: u32, rows: u32, strength: f32, color: &[u8], matrix: usize) -> Layer {
    Layer::new(columns as i32, rows as i32, strength, colors(color)[0], Matrix::ALL[matrix])
}

#[wasm_bindgen]
pub fn hover(kind: usize, columns: u32, rows: u32, t: f32, x: f32, y: f32, level: f32, strength: f32, color: &[u8], matrix: usize) -> Vec<u8> {
    bytes(lively::hover(Hover::ALL[kind], &layer(columns, rows, strength, color, matrix), t, (x, y), level))
}

#[wasm_bindgen]
pub fn press(kind: usize, columns: u32, rows: u32, t: f32, x: f32, y: f32, seed: i32, strength: f32, color: &[u8], matrix: usize) -> Vec<u8> {
    bytes(lively::press(Press::ALL[kind], &layer(columns, rows, strength, color, matrix), t, (x, y), seed))
}

#[wasm_bindgen]
pub fn idle(kind: usize, columns: u32, rows: u32, t: f32, strength: f32, color: &[u8], cell: f32, matrix: usize) -> Vec<u8> {
    bytes(lively::idle(Idle::ALL[kind], &layer(columns, rows, strength, color, matrix), t, cell))
}

#[wasm_bindgen]
pub fn shader(field: &str, columns: u32, rows: u32, time: f32, params: &[f32], palette: &[u8], matrix: usize, levels: u32) -> Vec<u8> {
    let Some(found) = field::lookup(field) else { return Vec::new() };
    let palette = colors(palette);
    bytes(field::render(&Shader {
        field: found.as_ref(),
        columns,
        rows,
        time,
        params,
        palette: &palette,
        matrix: Matrix::ALL[matrix],
        levels,
    }))
}

#[wasm_bindgen]
pub fn card_glow(columns: u32, rows: u32, corner: usize, reach: f32, strength: f32, boost: f32, rim: bool, radius: f32, variant: usize, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = GlowOptions {
        glow: Glow::ALL[corner],
        reach,
        strength,
        boost,
        rim,
        radius,
        fill: FillVariant::ALL[variant],
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(glow::card_glow(columns, rows, &options))
}

#[wasm_bindgen]
pub fn card_sparkles(columns: u32, rows: u32, cell: f32, tick: f32, corner: usize, reach: f32, color: &[u8]) -> Vec<u8> {
    let layer = layer(columns, rows, 1.0, color, 1);
    bytes(lively::glow_sparkles(&layer, cell, tick, Glow::ALL[corner], reach))
}

#[wasm_bindgen]
pub fn picture(pixels: &[u8], width: u32, height: u32, columns: u32, rows: u32, method: usize, threshold: f32, palette: &[u8]) -> Vec<u8> {
    let source = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(pixels, width, height));
    let options = render::PictureOptions { method: Method::ALL[method], threshold, palette: colors(palette) };
    bytes(render::picture(&source, columns, rows, &options))
}

#[wasm_bindgen]
pub fn reveal(pixels: &[u8], width: u32, height: u32, columns: u32, rows: u32, method: usize, threshold: f32, palette: &[u8], coarsest: u32, progress: f32, to_photo: bool) -> Vec<u8> {
    use dither_slint::reveal::{Reveal, RevealEnd};
    let source = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(pixels, width, height));
    let options = render::PictureOptions { method: Method::ALL[method], threshold, palette: colors(palette) };
    let end = if to_photo { RevealEnd::Photo } else { RevealEnd::Dither };
    bytes(dither_slint::reveal::reveal(&source, columns, rows, &options, &Reveal { coarsest, progress, end }))
}

#[wasm_bindgen]
pub fn logo(pixels: &[u8], width: u32, height: u32, columns: u32, rows: u32, fill: usize, matrix: usize, color: &[u8], original: bool, effect: usize, sweep: f32, outline: bool, floor: f32) -> Vec<u8> {
    use dither_slint::logo::{LogoEffect, LogoOptions, LogoSource, LogoTone};
    let source = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(pixels, width, height));
    let options = LogoOptions {
        source: LogoSource::Alpha,
        tone: if original { LogoTone::Original } else { LogoTone::Mono },
        fill: FillVariant::ALL[fill],
        outline,
        floor,
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
        effect: [LogoEffect::None, LogoEffect::Shimmer, LogoEffect::Scan][effect],
        sweep,
    };
    bytes(dither_slint::logo::logo(&source, columns, rows, &options))
}

#[wasm_bindgen]
pub fn treemap_layout(values: &[f32], remainder: i32, capacity: f32, progress: f32) -> Vec<f32> {
    let layout = dither_slint::treemap::layout(values, usize::try_from(remainder).ok(), capacity, progress);
    let mut flat = vec![layout.used];
    for tile in layout.tiles {
        flat.extend([tile.rect.x, tile.rect.y, tile.rect.w, tile.rect.h, tile.value]);
    }
    flat
}

#[wasm_bindgen]
pub fn treemap_format(value: f32) -> String {
    dither_slint::treemap::format_value(value)
}

fn chart_of(series: &[f32], lines: usize, width: f32, height: f32, flags: u8, dashes: &[u8]) -> (dither_slint::chart::Chart, Vec<dither_slint::chart::Series>) {
    let count = series.len() / lines.max(1);
    let stack = [dither_slint::chart::Stack::None, dither_slint::chart::Stack::Stacked, dither_slint::chart::Stack::Percent][usize::from(flags >> 3 & 3)];
    let chart = dither_slint::chart::Chart { band: flags & 32 != 0, stack, count, width, height, axes: flags & 1 != 0, zero: flags & 2 != 0, smooth: flags & 4 != 0 };
    let lines = series
        .chunks(count.max(1))
        .enumerate()
        .map(|(index, values)| dither_slint::chart::Series { values: values.to_vec(), low: Vec::new(), high: Vec::new(), dashed: dashes.get(index).is_some_and(|&dashed| dashed != 0) })
        .collect();
    (chart, lines)
}

#[wasm_bindgen]
pub fn chart_layout(series: &[f32], band: &[f32], lines: usize, width: f32, height: f32, flags: u8, dashes: &[u8], unit: &str) -> String {
    use dither_slint::chart;
    let (chart, lines) = chart_of(series, lines, width, height, flags, dashes);
    let edges = chart::edges(&lines, chart.stack);
    let scaled = if chart.stack == chart::Stack::None { series.iter().chain(band).copied().collect() } else { chart::scale_values(&lines, &edges, chart.stack) };
    let geometry = chart::geometry(&chart, &scaled, unit);
    let plot = geometry.plot;
    let ticks: Vec<String> = geometry
        .scale
        .ticks
        .iter()
        .zip(&geometry.tick_labels)
        .map(|(&tick, label)| format!("[{:.1},\"{label}\"]", chart::y_at(&plot, &geometry.scale, tick)))
        .collect();
    let labels: Vec<String> = chart::labels(&plot, chart.count).iter().map(|label| format!("[{:.1},{},{}]", label.x, label.index, label.align)).collect();
    let columns = chart.band.then(|| chart::bars(&chart, &geometry, &lines));
    let paths: Vec<String> = lines
        .iter()
        .zip(&edges)
        .enumerate()
        .map(|(position, (line, edges))| {
            let tops: Vec<f32> = edges.iter().map(|edge| edge.high).collect();
            let points = match &columns {
                Some(columns) => chart::bar_tops(&chart, &geometry, columns, position),
                None => chart::points(&plot, &geometry.scale, &tops, chart.count),
            };
            let curve = chart::segments(&points, chart.smooth);
            let path = if columns.is_some() { String::new() } else if line.dashed { chart::dashed_path(&curve) } else { chart::path(&points, &curve) };
            let points: Vec<String> = points.iter().map(|point| format!("[{:.1},{:.1}]", point.x, point.y)).collect();
            format!("{{\"path\":\"{path}\",\"points\":[{}]}}", points.join(","))
        })
        .collect();
    let totals = chart::totals(&chart, &lines);
    let listed: Vec<String> = totals.iter().map(|total| format!("{total:.2}")).collect();
    format!(
        "{{\"peak\":{},\"totals\":[{}],\"plot\":[{:.1},{:.1},{:.1},{:.1}],\"ticks\":[{}],\"labels\":[{}],\"lines\":[{}]}}",
        chart::max_index(&totals),
        listed.join(","),
        plot.left,
        plot.right,
        plot.top,
        plot.bottom,
        ticks.join(","),
        labels.join(","),
        paths.join(",")
    )
}

#[wasm_bindgen]
pub fn chart_band(series: &[f32], band: &[f32], lines: usize, low: &[f32], high: &[f32], width: f32, height: f32, flags: u8, index: usize, cell: f32, level: f32, matrix: usize, color: &[u8]) -> Vec<u8> {
    use dither_slint::chart;
    let (chart, mut lines) = chart_of(series, lines, width, height, flags, &[]);
    lines[index].low = low.to_vec();
    lines[index].high = high.to_vec();
    let values: Vec<f32> = series.iter().chain(band).copied().collect();
    let geometry = chart::geometry(&chart, &values, "");
    let options = chart::Band { chart: &chart, series: &lines[index], geometry: &geometry, cell, level, matrix: Matrix::ALL[matrix], color: colors(color)[0] };
    bytes(chart::band(&options))
}

#[wasm_bindgen]
pub fn chart_area(series: &[f32], lines: usize, width: f32, height: f32, flags: u8, colors: &[u8], cell: f32, level: f32, fill: usize, matrix: usize) -> Vec<u8> {
    use dither_slint::chart;
    let (chart, lines) = chart_of(series, lines, width, height, flags, &[]);
    let edges = chart::edges(&lines, chart.stack);
    let geometry = chart::geometry(&chart, &chart::scale_values(&lines, &edges, chart.stack), "");
    let colors = self::colors(colors);
    let options = chart::Area { chart: &chart, series: &lines, geometry: &geometry, colors: &colors, cell, level, matrix: Matrix::ALL[matrix], fill: FillVariant::ALL[fill] };
    bytes(chart::area(&options))
}

#[wasm_bindgen]
pub fn chart_bars(series: &[f32], lines: usize, width: f32, height: f32, flags: u8, colors: &[u8], cell: f32, level: f32, fill: usize, matrix: usize, solid: i32) -> Vec<u8> {
    use dither_slint::chart;
    let (chart, lines) = chart_of(series, lines, width, height, flags, &[]);
    let edges = chart::edges(&lines, chart.stack);
    let geometry = chart::geometry(&chart, &chart::scale_values(&lines, &edges, chart.stack), "");
    let columns = chart::bars(&chart, &geometry, &lines);
    let colors = self::colors(colors);
    let options = chart::BarFill { chart: &chart, geometry: &geometry, bars: &columns, colors: &colors, cell, level, matrix: Matrix::ALL[matrix], fill: FillVariant::ALL[fill], solid: usize::try_from(solid).ok() };
    bytes(chart::bars_image(&options))
}

#[wasm_bindgen]
pub fn chart_readout(series: &[f32], lines: usize, flags: u8, index: usize, unit: &str) -> String {
    use dither_slint::chart;
    let (chart, lines) = chart_of(series, lines, 0.0, 0.0, flags, &[]);
    let edges = chart::edges(&lines, chart.stack);
    chart::readout(&lines, &edges, index, unit, chart.stack).join("|")
}

#[wasm_bindgen]
pub fn chart_format(value: f32, unit: &str) -> String {
    dither_slint::chart::format_value(value, unit)
}

fn donut_of(values: &[f32], fills: &[u8], colors: &[u8], size: f32, cell: f32, flags: u8, thickness: f32, gap: f32) -> (dither_slint::donut::Spec, Vec<dither_slint::donut::Slice>) {
    use dither_slint::donut::{Mode, Slice, Spec};
    let spec = Spec { size, cell, mode: if flags & 1 != 0 { Mode::Pie } else { Mode::Donut }, thickness, gap, matrix: Matrix::Bayer4 };
    let colors = colors_from(colors);
    let slices = values.iter().enumerate().map(|(index, &value)| Slice { value, color: colors[index % colors.len().max(1)], fill: FillVariant::ALL[usize::from(fills.get(index).copied().unwrap_or(0))] }).collect();
    (spec, slices)
}

fn colors_from(rgba: &[u8]) -> Vec<Color> {
    colors(rgba)
}

#[wasm_bindgen]
pub fn donut_layout(values: &[f32], size: f32, cell: f32, flags: u8, thickness: f32, gap: f32) -> String {
    let (spec, slices) = donut_of(values, &[], &[0, 0, 0, 255], size, cell, flags, thickness, gap);
    let layout = dither_slint::donut::layout(&spec, &slices);
    let shares: Vec<String> = layout.arcs.iter().map(|arc| format!("{:.5}", arc.share)).collect();
    let middles: Vec<String> = layout.arcs.iter().map(|arc| format!("{:.5}", (arc.from + arc.to) / 2.0)).collect();
    format!("{{\"side\":{},\"outer\":{:.2},\"inner\":{:.2},\"total\":{},\"shares\":[{}],\"middles\":[{}]}}", layout.side, layout.outer, layout.inner, layout.total, shares.join(","), middles.join(","))
}

#[wasm_bindgen]
pub fn donut_image(values: &[f32], fills: &[u8], colors: &[u8], size: f32, cell: f32, flags: u8, thickness: f32, gap: f32, sweep: f32, focus: i32) -> Vec<u8> {
    let (spec, slices) = donut_of(values, fills, colors, size, cell, flags, thickness, gap);
    bytes(dither_slint::donut::image(&spec, &slices, sweep, usize::try_from(focus).ok()))
}

#[wasm_bindgen]
pub fn donut_hit(values: &[f32], size: f32, cell: f32, flags: u8, thickness: f32, gap: f32, x: f32, y: f32) -> i32 {
    let (spec, slices) = donut_of(values, &[], &[0, 0, 0, 255], size, cell, flags, thickness, gap);
    dither_slint::donut::hit(&spec, &slices, x, y).map_or(-1, |index| index as i32)
}

#[wasm_bindgen]
pub fn donut_value(value: f32, unit: &str) -> String {
    dither_slint::donut::format_value(value, unit)
}

#[wasm_bindgen]
pub fn donut_share(share: f32) -> String {
    dither_slint::donut::format_share(share)
}

fn heatmap_of(spec: &[f32], dates: &str, values: &[f32], grid: &[f32], columns: &str, rows: &str) -> dither_slint::heatmap::Input {
    use dither_slint::heatmap::{Input, Mode};
    let lines = |text: &str| -> Vec<String> { text.split('\n').filter(|line| !line.is_empty()).map(String::from).collect() };
    let (rows, columns) = (lines(rows), lines(columns));
    let width = columns.len().max(1);
    Input {
        mode: if spec[0] == 1.0 { Mode::Matrix } else { Mode::Calendar },
        entries: lines(dates).into_iter().zip(values.iter().copied()).collect(),
        grid: grid.chunks(width).map(<[f32]>::to_vec).collect(),
        rows,
        columns,
        weeks: spec[1] as usize,
        week_start: spec[2] as usize,
        steps: spec[3] as usize,
        max: (spec[4] > 0.0).then_some(spec[4]),
    }
}

fn heatmap_shown(spec: &[f32], input: &dither_slint::heatmap::Input) -> (dither_slint::heatmap::Model, dither_slint::heatmap::Fit) {
    use dither_slint::heatmap;
    let full = heatmap::model(input);
    let widest = if input.mode == heatmap::Mode::Matrix { full.column_labels.iter().map(|(_, text)| text.chars().count()).max().unwrap_or(0) } else { 0 };
    let fit = heatmap::fit(spec[5], spec[6], &full, widest);
    let shown = heatmap::shifted(&full, full.columns - fit.columns);
    (shown, fit)
}

#[wasm_bindgen]
pub fn heatmap_layout(spec: &[f32], dates: &str, values: &[f32], grid: &[f32], columns: &str, rows: &str) -> String {
    let input = heatmap_of(spec, dates, values, grid, columns, rows);
    let (model, fit) = heatmap_shown(spec, &input);
    let labels = |labels: &[(usize, String)]| labels.iter().map(|(position, text)| format!("[{position},\"{text}\"]")).collect::<Vec<_>>().join(",");
    format!(
        "{{\"cell\":{},\"columns\":{},\"rows\":{},\"left\":{},\"labelEvery\":{},\"peak\":{},\"columnLabels\":[{}],\"rowLabels\":[{}]}}",
        fit.cell,
        model.columns,
        model.rows,
        fit.left,
        fit.label_every,
        model.peak,
        labels(&model.column_labels),
        labels(&model.row_labels)
    )
}

#[wasm_bindgen]
pub fn heatmap_image(spec: &[f32], dates: &str, values: &[f32], grid: &[f32], columns: &str, rows: &str, color: &[u8], foreground: &[u8], focus: i32, reveal: f32) -> Vec<u8> {
    use dither_slint::heatmap;
    let input = heatmap_of(spec, dates, values, grid, columns, rows);
    let (model, _) = heatmap_shown(spec, &input);
    let options = heatmap::Paint { steps: heatmap::steps(input.steps), color: colors(color)[0], foreground: colors(foreground)[0], focus: usize::try_from(focus).ok(), reveal };
    bytes(heatmap::image(&model, &options))
}

#[wasm_bindgen]
pub fn heatmap_readout(spec: &[f32], dates: &str, values: &[f32], grid: &[f32], columns: &str, rows: &str, column: i32, row: i32, unit: &str, units: &str) -> String {
    use dither_slint::heatmap;
    let input = heatmap_of(spec, dates, values, grid, columns, rows);
    let (model, _) = heatmap_shown(spec, &input);
    heatmap::square_at(&model, column, row).map_or_else(String::new, |square| heatmap::readout(square, unit, units))
}

#[wasm_bindgen]
pub fn heatmap_swatch(bucket: usize, steps: usize, color: &[u8], foreground: &[u8]) -> Vec<u8> {
    use dither_slint::heatmap;
    bytes(heatmap::swatch(bucket, heatmap::steps(steps), colors(color)[0], colors(foreground)[0]))
}

#[wasm_bindgen]
pub fn pixel_area_side(pixels: f32, cell: f32) -> usize {
    dither_slint::pixel_area::side(pixels, cell)
}

#[wasm_bindgen]
pub fn pixel_area_image(values: &[f32], spec: &[f32], fill: usize, color: &[u8], marker: i32, stars: bool, winking: bool) -> Vec<u8> {
    use dither_slint::pixel_area::{Area, image};
    let area = Area {
        columns: spec[0] as usize,
        rows: spec[1] as usize,
        min: spec[2],
        max: (spec[3] != 0.0).then_some(spec[3]),
        fill: FillVariant::ALL[fill],
        color: colors(color)[0],
        reveal: spec[4],
        intensity: spec[5],
        marker: usize::try_from(marker).ok(),
        stars,
        winking,
        tick: spec[6],
    };
    bytes(image(values, &area))
}

#[wasm_bindgen]
pub fn pixel_area_readout(label: &str, value: f32) -> String {
    dither_slint::pixel_area::readout(label, value)
}

fn radar_of(values: &[f32], axes: usize, fills: &[u8], colors: &[u8], spec: &[f32]) -> (Vec<dither_slint::radar::Series>, dither_slint::radar::Spec) {
    use dither_slint::radar::{Grid, Series, Spec};
    let palette = colors_from(colors);
    let series = values
        .chunks(axes.max(1))
        .enumerate()
        .map(|(index, line)| Series { values: line.to_vec(), color: palette[index % palette.len().max(1)], fill: FillVariant::ALL[usize::from(fills.get(index).copied().unwrap_or(0))] })
        .collect();
    let spec = Spec { side: spec[0], cell: spec[1], max: (spec[2] > 0.0).then_some(spec[2]), rings: spec[3] as usize, grid: if spec[4] == 1.0 { Grid::Circle } else { Grid::Polygon }, matrix: Matrix::Bayer4 };
    (series, spec)
}

#[wasm_bindgen]
pub fn radar_layout(axes: &str, values: &[f32], fills: &[u8], spec: &[f32]) -> String {
    use dither_slint::radar;
    let labels: Vec<String> = axes.split('\n').filter(|line| !line.is_empty()).map(String::from).collect();
    let (series, spec) = radar_of(values, labels.len(), fills, &[0, 0, 0, 255], spec);
    let layout = radar::layout(&labels, &series, &spec);
    let point = |vertex: &radar::Vec2| format!("[{:.2},{:.2}]", vertex.x, vertex.y);
    let rings: Vec<String> = layout.rings.iter().map(|ring| format!("\"{ring}\"")).collect();
    let spokes: Vec<String> = layout.spokes.iter().map(point).collect();
    let texts: Vec<String> = layout.labels.iter().zip(&labels).map(|(label, text)| format!("[{:.2},{:.2},\"{text}\",{},{:.3}]", label.at.x, label.at.y, label.align, label.line)).collect();
    let shapes: Vec<String> = layout.vertices.iter().map(|vertices| format!("[{}]", vertices.iter().map(point).collect::<Vec<_>>().join(","))).collect();
    format!(
        "{{\"radius\":{:.2},\"top\":{},\"rings\":[{}],\"spokes\":[{}],\"labels\":[{}],\"shapes\":[{}]}}",
        layout.radius,
        layout.top,
        rings.join(","),
        spokes.join(","),
        texts.join(","),
        shapes.join(",")
    )
}

#[wasm_bindgen]
pub fn radar_image(axes: usize, values: &[f32], fills: &[u8], colors: &[u8], spec: &[f32], scale: f32, focus: i32) -> Vec<u8> {
    let (series, spec) = radar_of(values, axes, fills, colors, spec);
    bytes(dither_slint::radar::image(axes, &series, &spec, scale, usize::try_from(focus).ok()))
}

#[wasm_bindgen]
pub fn radar_axis(axes: usize, x: f32, y: f32, side: f32) -> i32 {
    dither_slint::radar::axis_at(axes.max(1), dither_slint::radar::Vec2 { x, y }, side).map_or(-1, |axis| axis as i32)
}

#[wasm_bindgen]
pub fn text_cell(font_size: f32) -> f32 {
    dither_slint::text::cell_for(font_size)
}

#[wasm_bindgen]
pub fn text_ink(columns: u32, rows: u32, spec: &[f32], backdrop: &[u8], time: f32) -> Vec<u8> {
    use dither_slint::text::{Ink, InkFill, ink_overlay};
    let fill = [InkFill::Gradient, InkFill::Noise, InkFill::Dotted, InkFill::Hatched, InkFill::Solid][spec[0] as usize];
    let ink = Ink { fill, direction: spec[1], scale: spec[2], from: spec[3], to: spec[4], ghost: (spec[5] >= 0.0).then_some(spec[5]), matrix: Matrix::Bayer4 };
    bytes(ink_overlay(columns, rows, &ink, time, colors(backdrop)[0]))
}

fn dissolve_of(spec: &[f32]) -> dither_slint::text::Dissolve {
    use dither_slint::text::{Dissolve, DissolveFill};
    Dissolve {
        duration: spec[0],
        stagger: spec[1],
        delay: spec[2],
        matrix: Matrix::Bayer4,
        fill: [DissolveFill::Solid, DissolveFill::Gradient, DissolveFill::Dotted, DissolveFill::Hatched][spec[3] as usize],
        level: spec[4],
        showing: spec[5] != 0.0,
    }
}

#[wasm_bindgen]
pub fn dissolve_units(text: &str, word: bool) -> String {
    let units: Vec<String> = dither_slint::text::units(text, word).into_iter().map(|(unit, index)| format!("[{:?},{}]", unit, index.map_or(-1, |index| index as i32))).collect();
    format!("[{}]", units.join(","))
}

#[wasm_bindgen]
pub fn dissolve_ranks(count: usize, order: usize) -> Vec<f32> {
    use dither_slint::text::DissolveOrder;
    dither_slint::text::ranks(count, [DissolveOrder::Start, DissolveOrder::End, DissolveOrder::Center, DissolveOrder::Edges, DissolveOrder::Random][order])
}

#[wasm_bindgen]
pub fn dissolve_step(elapsed: f32, rank: f32, spec: &[f32]) -> usize {
    dither_slint::text::dissolve_step(elapsed, rank, &dissolve_of(spec))
}

#[wasm_bindgen]
pub fn dissolve_overlay(columns: u32, rows: u32, step: usize, spec: &[f32], backdrop: &[u8]) -> Vec<u8> {
    bytes(dither_slint::text::dissolve_overlay(columns, rows, step, &dissolve_of(spec), colors(backdrop)[0]))
}

fn scramble_of(spec: &[f32]) -> dither_slint::text::Scramble {
    use dither_slint::text::{Scramble, ScrambleMode};
    Scramble { mode: [ScrambleMode::Dither, ScrambleMode::Blocks, ScrambleMode::Glyphs][spec[0] as usize], duration: spec[1], stagger: spec[2], delay: spec[3], rate: spec[4] }
}

#[wasm_bindgen]
pub fn scramble_slots(text: &str, from: &str, keep_same: bool, elapsed: f32, spec: &[f32], characters: &str) -> String {
    use dither_slint::text;
    let pool = text::glyph_pool(text, characters);
    let slots: Vec<String> = text::slots(text, from, keep_same, elapsed, &scramble_of(spec), &pool)
        .into_iter()
        .map(|slot| format!("[{:?},{},{},{:?}]", slot.character, slot.phase as i32, slot.level, slot.glyph))
        .collect();
    format!("[{}]", slots.join(","))
}

#[wasm_bindgen]
pub fn scramble_block(columns: u32, rows: u32, level: i32, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::text::block_overlay(columns, rows, level, colors(color)[0]))
}

#[wasm_bindgen]
pub fn rule(
    columns: u32,
    rows: u32,
    vertical: bool,
    fade_start: bool,
    fade_end: bool,
    fade: f32,
    strength: f32,
    variant: usize,
    matrix: usize,
    color: &[u8],
) -> Vec<u8> {
    let options = render::RuleOptions {
        vertical,
        fade_start,
        fade_end,
        fade,
        strength,
        variant: FillVariant::ALL[variant],
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(render::rule(columns, rows, &options))
}

#[wasm_bindgen]
pub fn slider_fill(columns: u32, rows: u32, end: i32, hot: bool, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = render::SliderOptions {
        start: 0,
        end,
        both: false,
        hot,
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(render::slider_fill(columns, rows, &options))
}

#[wasm_bindgen]
pub fn tabs_indicator(
    columns: u32,
    rows: u32,
    cell: f32,
    variant: usize,
    matrix: usize,
    color: &[u8],
    from: &[f32],
    to: &[f32],
    progress: f32,
) -> Vec<u8> {
    use dither_slint::tabs::{BoxRect, Indicator, TabsVariant, tabs_indicator};
    let variant = if variant == 0 { TabsVariant::Pill } else { TabsVariant::Underline };
    let rect = |b: &[f32]| BoxRect::new(b[0], b[1], b[2], b[3]);
    bytes(tabs_indicator(&Indicator {
        columns,
        rows,
        cell,
        variant,
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
        from: rect(from),
        to: rect(to),
        progress,
    }))
}

#[wasm_bindgen]
pub fn avatar(name: &str, fill: usize, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::avatar::avatar(name, FillVariant::ALL[fill], colors(color)[0]))
}

#[wasm_bindgen]
pub fn avatar_hue(name: &str) -> usize {
    dither_slint::avatar::hue(name)
}

#[wasm_bindgen]
pub fn loader(variant: usize, height: f32, cell: f32, time: f32, matrix: usize, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::loader::loader(dither_slint::loader::Variant::ALL[variant], height, cell, time, Matrix::ALL[matrix], colors(color)[0]))
}

#[wasm_bindgen]
pub fn loader_grid(variant: usize, height: f32, cell: f32) -> Vec<u32> {
    let (columns, rows) = dither_slint::loader::grid(dither_slint::loader::Variant::ALL[variant], height, cell);
    vec![columns, rows]
}

#[wasm_bindgen]
pub fn skeleton(columns: u32, rows: u32, cell: f32, fill: usize, animation: usize, phase: f32, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = dither_slint::skeleton::SkeletonOptions {
        cell,
        fill: FillVariant::ALL[fill],
        animation: dither_slint::skeleton::Animation::ALL[animation],
        phase,
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(dither_slint::skeleton::skeleton(columns, rows, &options))
}

#[wasm_bindgen]
pub fn empty_state(illustration: usize, backdrop: bool, matrix: usize, color: &[u8]) -> Vec<u8> {
    use dither_slint::empty::{Illustration, Layer, empty};
    let layer = if backdrop { Layer::Backdrop } else { Layer::Object };
    bytes(empty(Illustration::ALL[illustration], layer, Matrix::ALL[matrix], colors(color)[0]))
}

fn trend_frame(frame: &[f32]) -> dither_slint::trend::Frame {
    let [width, height, left, right, top, bottom, min, max] = frame[..8] else {
        unreachable!("a frame has eight numbers");
    };
    dither_slint::trend::Frame {
        width,
        height,
        margin: dither_slint::trend::Margin { left, right, top, bottom },
        domain: (min != max).then_some((min, max)),
    }
}

#[wasm_bindgen]
pub fn trend_area(values: &[f32], frame: &[f32], cell: f32, fill: usize, shading: usize, level: f32, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = dither_slint::trend::AreaOptions {
        fill: FillVariant::ALL[fill],
        shading: dither_slint::trend::Shading::ALL[shading],
        level,
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(dither_slint::trend::area(values, &trend_frame(frame), cell, &options))
}

#[wasm_bindgen]
pub fn trend_bloom(values: &[f32], frame: &[f32], cell: f32, area: bool, bloom: usize, color: &[u8]) -> Vec<u8> {
    let options = dither_slint::trend::BloomOptions { area, bloom: Bloom::ALL[bloom], color: colors(color)[0] };
    bytes(dither_slint::trend::bloom(values, &trend_frame(frame), cell, &options))
}

#[wasm_bindgen]
pub fn bloom_padding(bloom: usize, cell: f32) -> u32 {
    dither_slint::bloom::padding(Bloom::ALL[bloom], cell)
}

#[wasm_bindgen]
pub fn rect_bloom(columns: u32, rows: u32, cell: f32, bloom: usize, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::bloom::rect(columns, rows, cell, Bloom::ALL[bloom], colors(color)[0]))
}

#[wasm_bindgen]
pub fn trend_line(values: &[f32], frame: &[f32]) -> String {
    dither_slint::trend::path(&dither_slint::trend::points(values, &trend_frame(frame)))
}

#[wasm_bindgen]
pub fn trend_points(values: &[f32], frame: &[f32]) -> Vec<f32> {
    dither_slint::trend::points(values, &trend_frame(frame)).into_iter().flat_map(|point| [point.x, point.y]).collect()
}

#[wasm_bindgen]
pub fn trend_summary(label: &str, values: &[f32]) -> String {
    dither_slint::trend::summary(label, values)
}

#[wasm_bindgen]
pub fn format_compact(value: f32) -> String {
    dither_slint::stat::format_compact(value)
}

#[wasm_bindgen]
pub fn format_number(value: f32) -> String {
    dither_slint::stat::format_number(value)
}

#[wasm_bindgen]
pub fn stat_change(delta: f32, good: usize) -> Vec<String> {
    let change = dither_slint::stat::describe_change(delta, dither_slint::stat::Direction::ALL[good]);
    vec![change.text, format!("{:?}", change.direction).to_lowercase(), format!("{:?}", change.outcome).to_lowercase()]
}

fn uptime_day(date: &str, uptime: f32, note: &str, status: i32) -> dither_slint::uptime::Day {
    use dither_slint::uptime::{Day, Status};
    Day { date: date.into(), uptime, note: note.into(), status: usize::try_from(status).ok().map(|index| Status::ALL[index]) }
}

/// Status (index), status text, date, percentage and description of one day. A `status` below zero lets the uptime decide.
#[wasm_bindgen]
pub fn uptime_info(date: &str, uptime: f32, note: &str, status: i32) -> Vec<String> {
    use dither_slint::uptime::{Status, date as format_date, describe, percent, status as status_of};
    let day = uptime_day(date, uptime, note, status);
    let status = status_of(&day);
    let index = Status::ALL.iter().position(|each| *each == status).unwrap_or_default();
    let shown = if uptime < 0.0 { String::new() } else { percent(uptime) };
    vec![index.to_string(), status.text().into(), format_date(date), shown, describe(&day)]
}

#[wasm_bindgen]
pub fn uptime_overall(uptimes: &[f32]) -> String {
    let days: Vec<_> = uptimes.iter().map(|uptime| uptime_day("", *uptime, "", -1)).collect();
    dither_slint::uptime::overall(&days).map(dither_slint::uptime::percent).unwrap_or_default()
}

fn gauge(variant: usize, size: f32, segments: u32, ratio: f32, cell: f32) -> dither_slint::gauge::Gauge {
    dither_slint::gauge::Gauge { variant: dither_slint::gauge::Variant::ALL[variant], size, segments, ratio, cell }
}

/// Height, thickness, width of the range row and reveal time in milliseconds.
#[wasm_bindgen]
pub fn gauge_layout(variant: usize, size: f32, segments: u32, ratio: f32, cell: f32) -> Vec<f32> {
    let layout = dither_slint::gauge::layout(&gauge(variant, size, segments, ratio, cell));
    vec![layout.height, layout.thickness, layout.range_width, dither_slint::gauge::reveal(&layout)]
}

#[wasm_bindgen]
pub fn gauge_arc(variant: usize, size: f32, segments: u32, ratio: f32, cell: f32, style: &[u8], palette: &[u8], elapsed: f32) -> Vec<u8> {
    let [color, track] = colors(palette)[..2] else {
        unreachable!("a gauge has a colour and a track colour");
    };
    let arc = dither_slint::gauge::ArcStyle { fill: FillVariant::ALL[style[0] as usize], matrix: Matrix::ALL[style[1] as usize], color, track, elapsed };
    bytes(dither_slint::gauge::arc(&gauge(variant, size, segments, ratio, cell), &arc))
}

#[wasm_bindgen]
pub fn gauge_bloom(variant: usize, size: f32, segments: u32, ratio: f32, cell: f32, bloom: usize, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::gauge::glow(&gauge(variant, size, segments, ratio, cell), Bloom::ALL[bloom], colors(color)[0]))
}

#[wasm_bindgen]
pub fn format_decimals(value: f32, decimals: usize) -> String {
    dither_slint::stat::format_decimals(value, decimals)
}

fn number_roll(from: f32, to: f32, decimals: usize, timing: &[f32]) -> dither_slint::number::Roll {
    dither_slint::number::Roll { from, to, decimals, duration: timing[0], stagger: timing[1] }
}

/// Four numbers per glyph: 1 for a digit or 0 for anything else, its row, its tint and its character code.
#[wasm_bindgen]
pub fn number_glyphs(from: f32, to: f32, decimals: usize, timing: &[f32], elapsed: f32) -> Vec<f32> {
    use dither_slint::number::{Glyph, glyphs};
    glyphs(&number_roll(from, to, decimals, timing), elapsed)
        .into_iter()
        .flat_map(|glyph| match glyph {
            Glyph::Digit { row, tint } => [1.0, row, tint, 0.0],
            Glyph::Static(char) => [0.0, 0.0, 0.0, char as u32 as f32],
        })
        .collect()
}

#[wasm_bindgen]
pub fn number_length(from: f32, to: f32, decimals: usize, timing: &[f32]) -> f32 {
    dither_slint::number::length(&number_roll(from, to, decimals, timing))
}

#[wasm_bindgen]
pub fn number_edge(columns: u32, rows: u32, matrix: usize, color: &[u8]) -> Vec<u8> {
    bytes(dither_slint::number::edge(columns, rows, Matrix::ALL[matrix], colors(color)[0]))
}
