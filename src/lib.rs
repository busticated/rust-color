extern crate unicode_segmentation;
extern crate base16;

use unicode_segmentation::UnicodeSegmentation;
use std::vec::Vec;
use std::slice;

const RGB_MAX: f64 = 255.0;

#[derive(Debug, PartialEq)]
pub struct YIQ {
    pub y: f64,
    pub i: f64,
    pub q: f64
}

#[derive(Debug, PartialEq)]
pub struct HSL {
    pub h: f64,
    pub s: f64,
    pub l: f64
}

#[derive(Debug, PartialEq)]
pub struct HSLA {
    pub h: f64,
    pub s: f64,
    pub l: f64,
    pub a: f64
}

#[derive(Debug, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Debug, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64
}

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64

}

impl Color {
    pub fn new() -> Color {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0
        }
    }

    pub fn create(r: u8, g: u8, b: u8, a: f64) -> Color {
        let mut color = Color::new();
        color.r(r).g(g).b(b).a(a);
        color
    }

    pub fn from_hex(hex: String) -> Color {
        let hex = hex.trim().trim_start_matches('#');
        let letters = UnicodeSegmentation::graphemes(hex, true);
        let mut channels: Vec<String> = Vec::new();

        //letters.into_iter().flat_map(|s| s.chars().iter().chunks(2));

        for c in letters.take(6) {
            if let Some(last) = channels.last_mut() {
                if last.len() < 2 {
                    last.push_str(c)
                } else {
                    channels.push(c.to_string())
                }
            } else {
                channels.push(c.to_string())
            };
        }

        if channels.len() < 3 {
            return Color::create(0, 0, 0, 1.0)
        }

        let rgb: Vec<u8> = channels.iter().take(3)
            .map(|ch| match base16::decode(&ch) {
                Ok(v) => v[0],
                Err(_) => 0
            })
            .collect();

        Color::create(rgb[0], rgb[1], rgb[2], 1.0)
    }

    pub fn from_hsl(hue: f64, saturation: f64, lightness: f64) -> Color {
        fn to_rgb(n: f64) -> u8 { round(n * 255.0, 0.0) as u8 }
        let h = hue / 360.0;
        let s = to_decimal(saturation);
        let l = to_decimal(lightness);
        let r: f64;
        let g: f64;
        let b: f64;

        if s == 0.0 {
            r = l;
            g = l;
            b = l;
        } else {
            let q: f64;
            let p: f64;

            if l < 0.5 {
                q = l * (s + 1.0);
            } else {
                q = l + s - (l * s);
            }
            p = (l * 2.0) - q;
            r = hue_to_rgb(p, q, h + (1.0 / 3.0));
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - (1.0 / 3.0));
        }
        return Color::create(to_rgb(r), to_rgb(g), to_rgb(b), 1.0)
    }

    pub fn r(&mut self, r: u8) -> &mut Self {
        self.r = r;
        self
    }

    pub fn g(&mut self, g: u8) -> &mut Self {
        self.g = g;
        self
    }

    pub fn b(&mut self, b: u8) -> &mut Self {
        self.b = b;
        self
    }

    pub fn a(&mut self, a: f64) -> &mut Self {
        self.a = match a {
            a if a < 0.0 => 0.0,
            a if a > 1.0 => 1.0,
            _ => a
        };
        self
    }

    pub fn hex(&self) -> String {
        let mut hex = [&self.r, &self.g, &self.b].iter()
            .map(|&n| {
                let s = slice::from_ref(n);
                base16::encode_lower(&s)
            })
            .collect::<String>();

        hex.insert(0, '#');
        hex
    }

    // see: https://en.wikipedia.org/wiki/YIQ
    pub fn yiq(&self) -> YIQ {
        let r = self.r as f64;
        let g = self.g as f64;
        let b = self.b as f64;

        YIQ {
            y: (0.299 * r) + (0.587 * g) + (0.114 * b),
            i: (0.596 * r) + (-0.274 * g) + (-0.322 * b),
            q: (0.211 * r) + (-0.523 * g) + (0.312 * b)
        }
    }

    pub fn hsla(&self) -> HSLA {
        let r = self.r as f64 / RGB_MAX;
        let g = self.g as f64 / RGB_MAX;
        let b = self.b as f64 / RGB_MAX;
        let min = r.min(b.min(g));
        let max = r.max(b.max(g));
        let delta = max - min;
        let mut h:f64 = 0.0;
        let s:f64;
        let l:f64;

        if max == min {
            h = 0.0;
        } else if r == max{
            h = (g - b) / delta;
        } else if g == max {
            h = 2.0 + (b - r) / delta;
        } else if b == max {
            h = 4.0 + (r -g ) / delta;
        }

        h = (h * 60.0).min(360.0);

        if h < 0.0{
            h += 360.0;
        }

        l = (min + max) / 2.0;

        if max == min {
            s = 0.0;
        } else if l <= 0.5 {
            s = delta / (max + min);
        } else {
            s = delta / (2.0 - max - min);
        }

        HSLA {
            h: round(h, 0.0),
            s: round(s, 2.0),
            l: round(l, 2.0),
            a: self.a
        }
    }

    pub fn to_hsla_string(&self) -> String {
        let HSLA { h, s, l, a } = self.hsla();

        format!(
            "hsla({}, {}, {}, {})",
            h,
            to_pct(s),
            to_pct(l),
            to_decimal(a)
        )
    }

    pub fn to_rgba_string(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    pub fn is_light(&self) -> bool {
        let yiq = self.yiq();
        yiq.y >= 128.0
    }

    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }

    pub fn is_transparent(&self) -> bool {
        self.a == 0.0
    }
}


// UTILS //////////////////////////////////////////////////////////////////////
fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64{
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}


fn to_decimal(n: f64) -> f64 {
    match n > 1.0 {
        true => n / 100.0,
        false => n
    }
}

fn to_pct(n: f64) -> String {
    let pct = match n <= 1.0 {
        true => (n * 100.0).to_string(),
        false => n.to_string()
    };
    format!("{}%", pct)
}

fn round(n: f64, digits: f64) -> f64 {
    let places = (10.0f64).powf(digits);
    (n * &places).round() / &places
}


// TESTS //////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    struct Spec<T, U> { input: T, output: U }

    #[test]
    fn new() {
        let color = Color::new();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn from_hex() {
        let specs: Vec<Spec<String, Color>> = vec![
            Spec {
                input: String::from(""),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("🍿"),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("068"),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("🤪🥨👌"),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("#0089ff"),
                output: Color { r: 0, g: 137, b: 255, a: 1.0 }
            },
            Spec {
                input: String::from("#068000"),
                output: Color { r: 6, g: 128, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("068000"),
                output: Color { r: 6, g: 128, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("#WATNOPE"),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("🤪🤪🤪🤪🤪🤪"),
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: String::from("#FFFFFFFFFFFF"),
                output: Color { r: 255, g: 255, b: 255, a: 1.0 }
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let hex = String::from(input);
            let color = Color::from_hex(hex);

            println!(":::: Running Spec: {}", i);
            assert_eq!(color, *output);
        }
    }

    #[test]
    fn from_hsl() {
        let specs: Vec<Spec<HSL, Color>> = vec![
            Spec {
                input: HSL { h: 0.0, s: 0.0, l: 0.0 },
                output: Color { r: 0, g: 0, b: 0, a: 1.0 }
            },
            Spec {
                input: HSL { h: 208.0, s: 1.0, l: 0.5 },
                output: Color { r: 0, g: 136, b: 255, a: 1.0 }
            },
            Spec {
                input: HSL { h: 117.0, s: 1.0, l: 0.25 },
                output: Color { r: 6, g: 128, b: 0, a: 1.0 }
            },
            Spec {
                input: HSL { h: 0.0, s: 0.71, l: 0.53 },
                output: Color { r: 220, g: 50, b: 50, a: 1.0 }
            },
            Spec {
                input: HSL { h: 0.0, s: 0.0, l: 1.0 },
                output: Color { r: 255, g: 255, b: 255, a: 1.0 }
            },
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::from_hsl(input.h, input.s, input.l);

            println!(":::: Running Spec: {}", i);
            assert_eq!(color, *output);
        }
    }

    #[test]
    fn sets_r_channel() {
        let mut color = Color::new();

        assert_eq!(color.r, 0);

        color.r(10);

        assert_eq!(color.r, 10);
    }

    #[test]
    fn sets_g_channel() {
        let mut color = Color::new();

        assert_eq!(color.g, 0);

        color.g(10);

        assert_eq!(color.g, 10);
    }

    #[test]
    fn sets_b_channel() {
        let mut color = Color::new();

        assert_eq!(color.b, 0);

        color.b(10);

        assert_eq!(color.b, 10);
    }

    #[test]
    fn sets_a_channel() {
        let mut color = Color::new();

        assert_eq!(color.a, 1.0);

        color.a(10.0);

        assert_eq!(color.a, 1.0);

        color.a(-10.0);

        assert_eq!(color.a, 0.0);

        color.a(0.3);

        assert_eq!(color.a, 0.3);
    }

    #[test]
    fn hex() {
        let specs: Vec<Spec<RGBA, String >> = vec![
            Spec {
                input: RGBA { r: 0, g: 0, b: 0, a: 1.0 },
                output: String::from("#000000")
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 255, a: 1.0 },
                output: String::from("#ffffff")
            },
            Spec {
                input: RGBA { r: 0, g: 137, b: 255, a: 1.0 },
                output: String::from("#0089ff")
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::create(input.r, input.g, input.b, input.a);

            println!(":::: Running Spec: {}", i);
            assert_eq!(color.hex(), output[0..]);
        }
    }

    #[test]
    fn yiq() {
        let specs: Vec<Spec<RGBA, YIQ >> = vec![
            Spec {
                input: RGBA { r: 0, g: 0, b: 0, a: 1.0 },
                output: YIQ { y: 0.0, i: 0.0, q: 0.0 }
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 255, a: 1.0 },
                output: YIQ { y: 255.0, i: -0.000000000000014210854715202004, q: 0.0 }
            },
            Spec {
                input: RGBA { r: 0, g: 137, b: 255, a: 1.0 },
                output: YIQ { y: 109.489, i: -119.648, q: 7.909000000000006 }
            },
            Spec {
                input: RGBA { r: 255, g: 0, b: 0, a: 1.0 },
                output: YIQ { y: 76.24499999999999, i: 151.98, q: 53.805 }
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::create(input.r, input.g, input.b, input.a);
            let yiq = color.yiq();

            println!(":::: Running Spec: {}", i);
            assert_eq!(yiq, *output);
        }
    }

    #[test]
    fn hsla() {
        let specs: Vec<Spec<RGBA, HSLA>> = vec![
            Spec {
                input: RGBA { r: 0, g: 0, b: 0, a: 1.0 },
                output: HSLA { h: 0.0, s: 0.0, l: 0.0, a: 1.0 }
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 255, a: 1.0 },
                output: HSLA { h: 0.0, s: 0.0, l: 1.0, a: 1.0 }
            },
            Spec {
                input: RGBA { r: 0, g: 137, b: 255, a: 1.0 },
                output: HSLA { h: 208.0, s: 1.0, l: 0.5, a: 1.0 }
            },
            Spec {
                input: RGBA { r: 255, g: 0, b: 0, a: 0.5 },
                output: HSLA { h: 0.0, s: 1.0, l: 0.5, a: 0.5 }
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 0, a: 0.75 },
                output: HSLA { h: 60.0, s: 1.0, l: 0.5, a: 0.75 }
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::create(input.r, input.g, input.b, input.a);
            let hsla = color.hsla();

            println!(":::: Running Spec: {}", i);
            assert_eq!(hsla, *output);
        }
    }

    #[test]
    fn to_hsla_string() {
        let specs: Vec<Spec<RGBA, String>> = vec![
            Spec {
                input: RGBA { r: 0, g: 0, b: 0, a: 1.0 },
                output: String::from("hsla(0, 0%, 0%, 1)")
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 255, a: 1.0 },
                output: String::from("hsla(0, 0%, 100%, 1)")
            },
            Spec {
                input: RGBA { r: 0, g: 137, b: 255, a: 1.0 },
                output: String::from("hsla(208, 100%, 50%, 1)")
            },
            Spec {
                input: RGBA { r: 255, g: 0, b: 0, a: 0.5 },
                output: String::from("hsla(0, 100%, 50%, 0.5)")
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 0, a: 0.75 },
                output: String::from("hsla(60, 100%, 50%, 0.75)")
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::create(input.r, input.g, input.b, input.a);

            println!(":::: Running Spec: {}", i);
            assert_eq!(color.to_hsla_string(), output[0..]);
        }
    }

    #[test]
    fn to_rgba_string() {
        let specs: Vec<Spec<RGBA, String>> = vec![
            Spec {
                input: RGBA { r: 0, g: 0, b: 0, a: 1.0 },
                output: String::from("rgba(0, 0, 0, 1)")
            },
            Spec {
                input: RGBA { r: 255, g: 255, b: 255, a: 1.0 },
                output: String::from("rgba(255, 255, 255, 1)")
            },
            Spec {
                input: RGBA { r: 0, g: 137, b: 255, a: 1.0 },
                output: String::from("rgba(0, 137, 255, 1)")
            },
            Spec {
                input: RGBA { r: 255, g: 0, b: 0, a: 0.5 },
                output: String::from("rgba(255, 0, 0, 0.5)")
            }
        ];

        for (i, Spec { input, output }) in specs.iter().enumerate() {
            let color = Color::create(input.r, input.g, input.b, input.a);

            println!(":::: Running Spec: {}", i);
            assert_eq!(color.to_rgba_string(), output[0..]);
        }
    }

    #[test]
    fn is_light() {
        let specs: Vec<Spec<Color, bool>> = vec![
            Spec {
                input: Color { r: 0, g: 0, b: 0, a: 1.0 },
                output: false
            },
            Spec {
                input: Color { r: 255, g: 255, b: 255, a: 1.0 },
                output: true
            },
            Spec {
                input: Color { r: 0, g: 137, b: 255, a: 1.0 },
                output: false
            },
            Spec {
                input: Color { r: 255, g: 0, b: 0, a: 0.5 },
                output: false
            }
        ];

        for (i, Spec { input: color, output }) in specs.iter().enumerate() {
            println!(":::: Running Spec: {}", i);
            assert_eq!(color.is_light(), *output);
        }
    }

    #[test]
    fn is_dark() {
        let specs: Vec<Spec<Color, bool>> = vec![
            Spec {
                input: Color { r: 0, g: 0, b: 0, a: 1.0 },
                output: true
            },
            Spec {
                input: Color { r: 255, g: 255, b: 255, a: 1.0 },
                output: false
            },
            Spec {
                input: Color { r: 0, g: 137, b: 255, a: 1.0 },
                output: true
            },
            Spec {
                input: Color { r: 255, g: 0, b: 0, a: 0.5 },
                output: true
            }
        ];

        for (i, Spec { input: color, output }) in specs.iter().enumerate() {
            println!(":::: Running Spec: {}", i);
            assert_eq!(color.is_dark(), *output);
        }
    }

    #[test]
    fn is_transparent() {
        let specs: Vec<Spec<Color, bool>> = vec![
            Spec {
                input: Color { r: 0, g: 0, b: 0, a: 1.0 },
                output: false
            },
            Spec {
                input: Color { r: 0, g: 0, b: 0, a: 0.5 },
                output: false
            },
            Spec {
                input: Color { r: 0, g: 0, b: 0, a: 0.0 },
                output: true
            }
        ];

        for (i, Spec { input: color, output }) in specs.iter().enumerate() {
            println!(":::: Running Spec: {}", i);
            assert_eq!(color.is_transparent(), *output);
        }
    }
}
