use super::Vec3;
use std::{fs::{self, File}, io::{self, Write}, path::Path};

pub fn write_ppm(path: impl AsRef<Path>, width: u32, height: u32, pixels: &[Vec3]) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let mut file = File::create(path)?;
    write!(file, "P6\n{width} {height}\n255\n")?;
    for color in pixels {
        let c = color.clamp(0.0, 1.0);
        file.write_all(&[(c.x * 255.0) as u8, (c.y * 255.0) as u8, (c.z * 255.0) as u8])?;
    }
    Ok(())
}
