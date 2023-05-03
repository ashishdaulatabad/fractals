use colors_transform::{Color as OColor, Hsl, Rgb};
use pixel_canvas::Color;

pub fn build_color_array(max_iter: u32) -> Vec<Color> {
    let mut color_array: Vec<Color> = Vec::new();
    let fmx = max_iter as f64;
    for iter in 0..max_iter {
        let i = iter as f64;
        let fr = i / fmx;
        // let fr5 = fr3*fr3;
        let intensity: f32 = fr.sqrt() as f32;
        let quad_intensity: f32 = intensity.sqrt() as f32;
        let deg: f32 = 225 as f32;
        let hue: f32 = quad_intensity * 70.0 + 30.0;
        let rgb: Rgb = Hsl::from(deg, hue, quad_intensity * 60.0 + 40.0).to_rgb();
        color_array.push(Color {
            r: rgb.get_red() as u8,
            g: rgb.get_green() as u8,
            b: rgb.get_blue() as u8,
        });
    }
    color_array.push(Color { r: 0, g: 0, b: 0 });
    color_array
}
