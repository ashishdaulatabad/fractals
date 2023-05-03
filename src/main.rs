#![feature(stdsimd)]
use fractal::Fractal;
use pixel_canvas::{input::MouseState, Canvas};
mod color;
mod complex;
mod fractal;
mod polynomial;
mod utils;

fn main() {
    // The canvas will render for you at up to 60fps.
    let mut fractal: Fractal = Fractal::new()
        .set_window_dim(1600, 900)
        .set_num_threads(16)
        .set_max_iter(120)
        .set_prec(fractal::Precision::F32)
        .set_iset(fractal::InstructionSet::SSE)
        .set_fractal(fractal::FractalType::Julia)
        .set_pow(2);

    let (width, height) = fractal.get_dim();

    let canvas = Canvas::new(width as usize, height as usize)
        .title("Fractal")
        .show_ms(true)
        .state(MouseState::new())
        .input(MouseState::handle_input);

    canvas.render(move |mouse, image| {
        // to do: zoom in
        let x = mouse.x;
        let y = mouse.y;
        fractal.draw(image, x, y);
    });
}
