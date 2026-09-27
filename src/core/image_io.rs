use super::Vec3;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn write_ppm(path: &str, width: usize, height: usize, pixels: &[Vec3]) -> io::Result<()> {
    let expected_pixels = width
        .checked_mul(height)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "image dimensions overflow"))?;
    if pixels.len() != expected_pixels {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pixel count does not match image dimensions",
        ));
    }

    let path = Path::new(path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let mut writer = BufWriter::new(File::create(path)?);
    write!(writer, "P6\n{width} {height}\n255\n")?;

    for pixel in pixels {
        let color = pixel.clamp(0.0, 1.0);
        let rgb = [
            gamma_to_byte(color.x),
            gamma_to_byte(color.y),
            gamma_to_byte(color.z),
        ];
        writer.write_all(&rgb)?;
    }

    writer.flush()
}

fn gamma_to_byte(value: f64) -> u8 {
    (value.powf(1.0 / 2.2) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::gamma_to_byte;

    #[test]
    fn gamma_conversion_covers_byte_range() {
        assert_eq!(gamma_to_byte(0.0), 0);
        assert_eq!(gamma_to_byte(1.0), 255);
    }
}
