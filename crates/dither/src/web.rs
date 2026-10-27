use crate::*;
use wasm_bindgen::prelude::*;

fn config(matrix: &str, colors: &[u8], contrast: f32) -> Result<Dither, JsValue> {
    let matrix = match matrix {
        "bayer2" => Matrix::Bayer2,
        "bayer4" => Matrix::Bayer4,
        "bayer8" => Matrix::Bayer8,
        "blue-noise" => Matrix::BlueNoise,
        _ => return Err(JsValue::from_str("unknown matrix")),
    };
    if !colors.len().is_multiple_of(3) {
        return Err(JsValue::from_str("palette must contain RGB triples"));
    }
    let palette = Palette::new(colors.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect())
        .ok_or_else(|| JsValue::from_str("at least two colors required"))?;
    Ok(Dither {
        matrix,
        palette,
        contrast,
    })
}
#[wasm_bindgen]
pub fn render_field(
    width: usize,
    height: usize,
    preset: &str,
    time: f32,
    matrix: &str,
    colors: &[u8],
    contrast: f32,
) -> Result<Vec<u8>, JsValue> {
    let preset = Preset::from_slug(preset).ok_or_else(|| JsValue::from_str("unknown preset"))?;
    let raster = config(matrix, colors, contrast)?
        .render_preset(width, height, preset, time)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(raster.pixels().to_vec())
}
/// Dither caller-supplied tones, one per cell in row-major order.
#[wasm_bindgen]
pub fn render_tones(
    width: usize,
    height: usize,
    tones: &[f32],
    matrix: &str,
    colors: &[u8],
    contrast: f32,
) -> Result<Vec<u8>, JsValue> {
    if width.checked_mul(height) != Some(tones.len()) {
        return Err(JsValue::from_str(
            "tones must contain width × height values",
        ));
    }
    let raster = config(matrix, colors, contrast)?
        .render(width, height, |x, y| {
            let column = ((x * width as f32) as usize).min(width - 1);
            let row = ((y * height as f32) as usize).min(height - 1);
            tones[row * width + column]
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(raster.pixels().to_vec())
}
#[wasm_bindgen]
pub fn export_svg(
    width: usize,
    height: usize,
    preset: &str,
    time: f32,
    matrix: &str,
    colors: &[u8],
    contrast: f32,
) -> Result<String, JsValue> {
    let preset = Preset::from_slug(preset).ok_or_else(|| JsValue::from_str("unknown preset"))?;
    Ok(config(matrix, colors, contrast)?
        .render_preset(width, height, preset, time)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .svg())
}
#[wasm_bindgen]
pub fn convert_image(
    width: usize,
    height: usize,
    rgba: &[u8],
    colors: &[u8],
    method: &str,
) -> Result<Vec<u8>, JsValue> {
    let d = config("bayer8", colors, 1.)?;
    let method = match method {
        "floyd-steinberg" => ImageMethod::FloydSteinberg,
        "atkinson" => ImageMethod::Atkinson,
        "sierra-lite" => ImageMethod::SierraLite,
        "blue-noise" => ImageMethod::Ordered(Matrix::BlueNoise),
        "bayer4" => ImageMethod::Ordered(Matrix::Bayer4),
        "bayer8" => ImageMethod::Ordered(Matrix::Bayer8),
        _ => return Err(JsValue::from_str("unknown method")),
    };
    Ok(dither_image(width, height, rgba, d.palette, method)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .pixels()
        .to_vec())
}
#[wasm_bindgen]
pub fn catalog() -> String {
    Preset::ALL
        .iter()
        .map(|p| format!("{}|{}|{}", p.slug(), p.label(), p.group()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rgb(bytes: &[u8]) -> Result<[u8; 3], JsValue> {
    bytes
        .try_into()
        .map_err(|_| JsValue::from_str("colors must be RGB triples"))
}
fn places(flat: &[f32]) -> Vec<LatLon> {
    flat.chunks_exact(2).map(|p| (p[0], p[1])).collect()
}
/// A globe frame. `markers` is flat `[lat, lon, …]`; `arcs` is flat
/// `[from_lat, from_lon, to_lat, to_lon, …]`. Paper is transparent.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn render_globe(
    columns: usize,
    rows: usize,
    time: f32,
    speed: f32,
    tilt: f32,
    ink: &[u8],
    accent: &[u8],
    markers: &[f32],
    arcs: &[f32],
) -> Result<Vec<u8>, JsValue> {
    let globe = Globe {
        speed,
        tilt,
        markers: places(markers),
        arcs: arcs
            .chunks_exact(4)
            .map(|a| ((a[0], a[1]), (a[2], a[3])))
            .collect(),
        ..Globe::default()
    };
    Ok(globe
        .render(columns, rows, time, rgb(ink)?, rgb(accent)?)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .pixels()
        .to_vec())
}
/// A loader frame on transparent paper.
#[wasm_bindgen]
pub fn render_loader(
    variant: &str,
    columns: usize,
    rows: usize,
    time: f32,
    ink: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let variant =
        LoaderVariant::from_slug(variant).ok_or_else(|| JsValue::from_str("unknown loader"))?;
    Ok(variant
        .render(columns, rows, time, Matrix::Bayer4, rgb(ink)?)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .pixels()
        .to_vec())
}
/// The loader's grid for a height in cells, as `[columns, rows]`.
#[wasm_bindgen]
pub fn loader_grid(variant: &str, height: usize) -> Result<Vec<u32>, JsValue> {
    let variant =
        LoaderVariant::from_slug(variant).ok_or_else(|| JsValue::from_str("unknown loader"))?;
    let (c, r) = variant.grid(height);
    Ok(vec![c as u32, r as u32])
}

/// Chart marks. `values` is row-major, `series` rows of `count` values;
/// `fills` is a comma-separated slug per series. Paper is transparent.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)]
pub fn render_chart(
    kind: &str,
    columns: usize,
    rows: usize,
    values: &[f32],
    series: usize,
    fills: &str,
    stacked: bool,
    top: f32,
    progress: f32,
    colors: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let kind = match kind {
        "area" => PlotKind::Area,
        "line" => PlotKind::Line,
        "bar" => PlotKind::Bar,
        "donut" => PlotKind::Donut,
        "radar" => PlotKind::Radar,
        "heatmap" => PlotKind::Heatmap,
        _ => return Err(JsValue::from_str("unknown chart kind")),
    };
    let count = values.len().checked_div(series).unwrap_or(0);
    let data: Vec<Vec<f32>> = values
        .chunks(count.max(1))
        .take(series)
        .map(<[f32]>::to_vec)
        .collect();
    let fills: Vec<ChartFill> = fills
        .split(',')
        .enumerate()
        .map(|(i, f)| ChartFill::from_slug(f.trim()).unwrap_or(ChartFill::nth(i)))
        .collect();
    let colors: Vec<[u8; 3]> = colors.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let plot = Plot {
        kind,
        series: &data,
        fills: &fills,
        stacked,
        top,
        progress,
    };
    Ok(plot
        .render(columns, rows, &colors)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .pixels()
        .to_vec())
}

/// The avatar hue for a name, in degrees.
#[wasm_bindgen]
pub fn avatar_hue(name: &str) -> u16 {
    Avatar::from_name(name).hue
}
/// An avatar raster, `8 × scale` cells square, on transparent paper.
#[wasm_bindgen]
pub fn render_avatar(name: &str, scale: usize, ink: &[u8]) -> Result<Vec<u8>, JsValue> {
    Ok(Avatar::from_name(name)
        .render(scale, rgb(ink)?)
        .pixels()
        .to_vec())
}
