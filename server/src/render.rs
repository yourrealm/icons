//! Rasterising with resvg: SVG text to PNG, and to ICO with 16, 32 and 48 px
//! frames. No system tools.

use resvg::{tiny_skia, usvg};

fn raster(svg: &str, size: u32) -> Result<tiny_skia::Pixmap, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).ok_or("zero size")?;
    let s = size as f32 / tree.size().width();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(s, s),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

pub fn png(svg: &str, size: u32) -> Result<Vec<u8>, String> {
    raster(svg, size)?.encode_png().map_err(|e| e.to_string())
}

pub const ICO_SIZES: [u32; 3] = [16, 32, 48];

pub fn ico(svg: &str) -> Result<Vec<u8>, String> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in ICO_SIZES {
        let pixmap = raster(svg, size)?;
        let rgba: Vec<u8> = pixmap
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        let image = ico::IconImage::from_rgba_data(size, size, rgba);
        dir.add_entry(ico::IconDirEntry::encode(&image).map_err(|e| e.to_string())?);
    }
    let mut out = Vec::new();
    dir.write(&mut out).map_err(|e| e.to_string())?;
    Ok(out)
}
