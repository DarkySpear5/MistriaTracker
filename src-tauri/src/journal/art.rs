use super::catalog::JournalCatalog;
use crate::catalog::{AssetsZip, CatalogCacheDir, CatalogError};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    time::UNIX_EPOCH,
};
use tempfile::NamedTempFile;
use zip::ZipArchive;

pub fn prepare(
    source: &AssetsZip,
    cache: &CatalogCacheDir,
    catalog: &JournalCatalog,
) -> Result<PathBuf, CatalogError> {
    let metadata = fs::metadata(source.as_path())?;
    let stamp = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CatalogError::InvalidSource("invalid asset timestamp"))?
        .as_nanos();
    let directory = cache
        .as_path()
        .join(format!("journal-art-v3-{}-{stamp}", metadata.len()));
    if directory.join("complete").is_file() {
        return Ok(directory);
    }
    fs::create_dir_all(&directory)?;
    let sprites: BTreeSet<_> = catalog
        .entries
        .values()
        .map(|v| v.sprite.as_str())
        .chain(catalog.villagers.iter().map(|v| v.portrait.as_str()))
        .filter(|v| !v.is_empty())
        .collect();
    let mut zip = ZipArchive::new(File::open(source.as_path())?)?;
    for sprite in sprites {
        if !sprite
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || v == b'_')
        {
            continue;
        }
        let Some(path) = catalog.artwork.get(sprite) else {
            continue;
        };
        let mut entry = zip.by_name(path)?;
        if entry.size() > 4 * 1024 * 1024 {
            return Err(CatalogError::InvalidSource("artwork exceeds size limit"));
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if !data.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) {
            return Err(CatalogError::InvalidSource("artwork is not PNG"));
        }
        drop(entry);
        let meta_path = path.trim_end_matches(".png").to_owned() + ".meta.toml";
        let frame = if let Ok(mut meta) = zip.by_name(&meta_path) {
            if meta.size() > 64 * 1024 {
                return Err(CatalogError::InvalidSource(
                    "artwork metadata exceeds limit",
                ));
            }
            let mut text = String::new();
            meta.read_to_string(&mut text)?;
            let metadata: toml::Value = toml::from_str(&text)?;
            metadata
                .get("asset_properties")
                .and_then(|v| v.get("frame_size"))
                .and_then(toml::Value::as_array)
                .and_then(|v| {
                    let width = u32::try_from(v.first()?.as_integer()?).ok()?;
                    let height = u32::try_from(v.get(1)?.as_integer()?).ok()?;
                    Some((width, height))
                })
        } else {
            None
        };
        let data = first_frame(&data, frame, sprite.starts_with("spr_portrait_"))?;
        let mut file = NamedTempFile::new_in(&directory)?;
        file.write_all(&data)?;
        file.flush()?;
        file.persist(directory.join(format!("{sprite}.png")))
            .map_err(|e| e.error)?;
        fs::write(
            directory.join(format!("{sprite}_hidden.png")),
            silhouette(&data)?,
        )?;
    }
    fs::write(directory.join("complete"), b"journal-art-v3")?;
    Ok(directory)
}

/// Game animations are sheets; the journal displays only their first frame.
pub fn first_frame(
    bytes: &[u8],
    frame: Option<(u32, u32)>,
    trim: bool,
) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(std::io::Error::other)?;
    if reader.output_buffer_size() > 32 * 1024 * 1024 {
        return Err(std::io::Error::other("artwork dimensions exceed budget"));
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(std::io::Error::other)?;
    let (width, height) = frame.unwrap_or((info.width, info.height));
    if width == 0 || height == 0 || width > info.width || height > info.height {
        return Err(std::io::Error::other("invalid artwork frame"));
    }
    let channels = info.color_type.samples();
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let at = (y * info.width as usize + x) * channels;
            let p = &buffer[at..at + channels];
            let pixel = match info.color_type {
                png::ColorType::Rgba => [p[0], p[1], p[2], p[3]],
                png::ColorType::Rgb => [p[0], p[1], p[2], 255],
                png::ColorType::GrayscaleAlpha => [p[0], p[0], p[0], p[1]],
                png::ColorType::Grayscale => [p[0], p[0], p[0], 255],
                _ => return Err(std::io::Error::other("unexpanded artwork palette")),
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    let (mut left, mut top, mut right, mut bottom) = (0, 0, width, height);
    if trim {
        let mut bounds = (width, height, 0, 0);
        for y in 0..height {
            for x in 0..width {
                if rgba[((y * width + x) * 4 + 3) as usize] > 0 {
                    bounds.0 = bounds.0.min(x);
                    bounds.1 = bounds.1.min(y);
                    bounds.2 = bounds.2.max(x + 1);
                    bounds.3 = bounds.3.max(y + 1);
                }
            }
        }
        if bounds.2 > bounds.0 && bounds.3 > bounds.1 {
            (left, top, right, bottom) = bounds;
        }
    }
    let mut cropped = Vec::new();
    for y in top..bottom {
        cropped.extend_from_slice(
            &rgba[((y * width + left) * 4) as usize..((y * width + right) * 4) as usize],
        );
    }
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, right - left, bottom - top);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .map_err(std::io::Error::other)?
            .write_image_data(&cropped)
            .map_err(std::io::Error::other)?;
    }
    Ok(encoded)
}

/// Remove every RGB value before hidden artwork can cross the desktop boundary.
pub fn silhouette(bytes: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(std::io::Error::other)?;
    if reader.output_buffer_size() > 32 * 1024 * 1024 {
        return Err(std::io::Error::other("silhouette dimensions exceed budget"));
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(std::io::Error::other)?;
    let (channels, alpha) = match info.color_type {
        png::ColorType::Rgba => (4, Some(3)),
        png::ColorType::GrayscaleAlpha => (2, Some(1)),
        png::ColorType::Rgb => (3, None),
        png::ColorType::Grayscale => (1, None),
        _ => return Err(std::io::Error::other("unexpanded silhouette palette")),
    };
    let pixels: Vec<u8> = buffer[..info.buffer_size()]
        .chunks_exact(channels)
        .flat_map(|pixel| [0, 0, 0, alpha.map(|index| pixel[index]).unwrap_or(255)])
        .collect();
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, info.width, info.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
        writer
            .write_image_data(&pixels)
            .map_err(std::io::Error::other)?;
    }
    Ok(encoded)
}
