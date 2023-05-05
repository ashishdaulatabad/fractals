use colorsys::{Hsl, Rgb};
use pixel_canvas::Color;

pub fn build_color_array(max_iter: u32) -> Vec<Color> {
    let mut color_array: Vec<Color> = Vec::new();
    let fmx = max_iter as f64;
    for iter in 0..=max_iter {
        let i = iter as f64;
        let fr = i / fmx;
        let intensity: f32 = fr.sqrt() as f32;
        let quad_intensity: f32 = intensity.sqrt() as f32;
        let deg: f32 = 270_f32;
        let hue: f32 = quad_intensity * 100.0;
        let rgb: Rgb = Rgb::from(Hsl::from((deg - 12.0_f32, hue, quad_intensity * 80.0)));
        color_array.push(Color {
            r: rgb.red() as u8,
            g: rgb.green() as u8,
            b: rgb.blue() as u8,
        });
    }
    color_array
}
