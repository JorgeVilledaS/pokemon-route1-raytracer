use crate::core::Vec3;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec3>,
}

impl Texture {
    pub fn solid(color: Vec3) -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![color],
        }
    }

    pub fn load_ppm(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let data = fs::read(path)?;
        Self::decode_ppm(&data).map_err(|message| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {message}", path.display()),
            )
        })
    }

    pub fn sample(&self, u: f64, v: f64) -> Vec3 {
        let x = u.rem_euclid(1.0) * self.width as f64 - 0.5;
        let y = v.rem_euclid(1.0) * self.height as f64 - 0.5;
        let x0 = x.floor() as isize;
        let y0 = y.floor() as isize;
        let tx = x - x.floor();
        let ty = y - y.floor();

        let c00 = self.pixel_wrapped(x0, y0);
        let c10 = self.pixel_wrapped(x0 + 1, y0);
        let c01 = self.pixel_wrapped(x0, y0 + 1);
        let c11 = self.pixel_wrapped(x0 + 1, y0 + 1);
        let top = c00 * (1.0 - tx) + c10 * tx;
        let bottom = c01 * (1.0 - tx) + c11 * tx;

        top * (1.0 - ty) + bottom * ty
    }

    fn pixel_wrapped(&self, x: isize, y: isize) -> Vec3 {
        let x = x.rem_euclid(self.width as isize) as usize;
        let y = y.rem_euclid(self.height as isize) as usize;
        self.pixels[x + y * self.width]
    }

    fn decode_ppm(data: &[u8]) -> Result<Self, String> {
        let mut parser = PpmParser::new(data);
        let magic = parser.token()?.to_vec();
        let width = parser.parse_usize("width")?;
        let height = parser.parse_usize("height")?;
        let max_value = parser.parse_usize("maximum channel value")?;

        if width == 0 || height == 0 {
            return Err("dimensions must be positive".into());
        }
        if max_value == 0 || max_value > 255 {
            return Err("only PPM channel ranges 1..=255 are supported".into());
        }
        let channel_count = width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(3))
            .ok_or_else(|| "texture dimensions overflow usize".to_string())?;

        let channels = match magic.as_slice() {
            b"P3" => {
                let mut channels = Vec::with_capacity(channel_count);
                for _ in 0..channel_count {
                    let channel = parser.parse_usize("P3 channel")?;
                    if channel > max_value {
                        return Err("P3 channel exceeds declared maximum".into());
                    }
                    channels.push(channel as u8);
                }
                channels
            }
            b"P6" => {
                parser.consume_binary_separator()?;
                let remaining = &data[parser.position..];
                if remaining.len() < channel_count {
                    return Err("truncated P6 pixel data".into());
                }
                remaining[..channel_count].to_vec()
            }
            _ => return Err("expected P3 or P6 PPM magic number".into()),
        };

        let scale = max_value as f64;
        let pixels = channels
            .as_chunks::<3>()
            .0
            .iter()
            .map(|rgb| {
                Vec3::new(
                    f64::from(rgb[0]) / scale,
                    f64::from(rgb[1]) / scale,
                    f64::from(rgb[2]) / scale,
                )
            })
            .collect();

        Ok(Self {
            width,
            height,
            pixels,
        })
    }
}

struct PpmParser<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> PpmParser<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn token(&mut self) -> Result<&'a [u8], String> {
        self.skip_whitespace_and_comments();
        let start = self.position;
        while self.position < self.data.len()
            && !self.data[self.position].is_ascii_whitespace()
            && self.data[self.position] != b'#'
        {
            self.position += 1;
        }

        (start != self.position)
            .then_some(&self.data[start..self.position])
            .ok_or_else(|| "unexpected end of PPM header".to_string())
    }

    fn parse_usize(&mut self, label: &str) -> Result<usize, String> {
        let token = self.token()?;
        let text = std::str::from_utf8(token).map_err(|_| format!("invalid {label}"))?;
        text.parse().map_err(|_| format!("invalid {label}"))
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.position < self.data.len() && self.data[self.position].is_ascii_whitespace()
            {
                self.position += 1;
            }
            if self.position >= self.data.len() || self.data[self.position] != b'#' {
                break;
            }
            while self.position < self.data.len() && self.data[self.position] != b'\n' {
                self.position += 1;
            }
        }
    }

    fn consume_binary_separator(&mut self) -> Result<(), String> {
        if self.position >= self.data.len() || !self.data[self.position].is_ascii_whitespace() {
            return Err("P6 header must end with whitespace".into());
        }
        if self.data[self.position] == b'\r'
            && self.data.get(self.position + 1).copied() == Some(b'\n')
        {
            self.position += 2;
        } else {
            self.position += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Texture;
    use crate::core::Vec3;

    const EPSILON: f64 = 1.0e-12;

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
        assert!((actual.z - expected.z).abs() < EPSILON);
    }

    #[test]
    fn decodes_ascii_ppm_with_comments() {
        let ppm = b"P3\n# tiny texture\n2 1\n255\n255 0 0  0 255 0\n";
        let texture = Texture::decode_ppm(ppm).expect("valid P3 texture");

        assert_eq!((texture.width, texture.height), (2, 1));
        assert_eq!(texture.pixels[0], Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(texture.pixels[1], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn decodes_binary_ppm() {
        let ppm = b"P6\n1 1\n255\n\x20\x40\x80";
        let texture = Texture::decode_ppm(ppm).expect("valid P6 texture");

        assert_vec3_close(
            texture.pixels[0],
            Vec3::new(32.0 / 255.0, 64.0 / 255.0, 128.0 / 255.0),
        );
    }

    #[test]
    fn bilinear_sampling_wraps_on_both_axes() {
        let texture = Texture {
            width: 2,
            height: 2,
            pixels: vec![
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 1.0, 1.0),
            ],
        };

        assert_vec3_close(texture.sample(0.25, 0.25), texture.pixels[0]);
        assert_vec3_close(texture.sample(1.25, -0.75), texture.pixels[0]);
        assert_vec3_close(texture.sample(0.5, 0.5), Vec3::new(0.5, 0.5, 0.5));
    }
}
