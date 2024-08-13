use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Renderer {}
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, vram: &[u8]) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (i, &char_code) in vram.iter().enumerate() {
            let x = (i % 40) as i32 * 20;
            let y = (i / 40) as i32 * 24;

            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.fill_rect(Rect::new(x, y, 20, 24))?;

            if char_code != 0 {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.fill_rect(Rect::new(x + 2, y + 2, 16, 20))?;
            }
        }

        Ok(())
    }
}
