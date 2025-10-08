use crate::resources::provide_style_path;
use raylib::prelude::*;

// TODO: Menu Bar
// 1. Game
//   - new
//   - close
//   - resign
//   - save
//   - save as
//   - copy FEN
//   - copy PGN
//   - paste FEN

// 2. Tournament
//   - new
//   - stop
//   - Active Games
//   - results

// 3. Tools
//   - Settings
//   - Game Database

// 4. View
//   - (give common ways to add/remove UI elements)

// 5. Help
//   - About Cactus GUI

struct UIScaler {
    screen_width: f32,
    screen_height: f32,
    screen_ratio: f32,
}

impl UIScaler {
    fn new(screen_width: i32, screen_height: i32) -> Self {
        Self {
            screen_width: screen_width as f32,
            screen_height: screen_height as f32,
            screen_ratio: screen_width as f32 / screen_height as f32,
        }
    }

    fn scale_w(&self, scale: f32) -> i32 {
        (self.screen_width * scale) as i32
    }

    fn scale_h(&self, scale: f32) -> i32 {
        (self.screen_height * scale) as i32
    }

    /// Convert from a height-based fraction to a width-based pixel size
    fn scale_w_from_h(&self, h_scale: f32) -> i32 {
        (self.screen_height * h_scale * self.screen_ratio) as i32
    }

    /// Convert from a width-based fraction to a height-based pixel size
    fn scale_h_from_w(&self, w_scale: f32) -> i32 {
        (self.screen_width * w_scale / self.screen_ratio) as i32
    }
}

pub fn build_interface() {
    let (mut rl, thread) = raylib::init().size(1300, 900).title("Cactus GUI").build();

    // Load the dark style
    unsafe {
        let path = std::ffi::CString::new(provide_style_path().to_str().unwrap()).unwrap();
        raylib::ffi::GuiLoadStyle(path.as_ptr());
    }

    let bg_color = unsafe { Color::get_color(raylib::ffi::GuiGetStyle(0, 19) as u32) };

    while !rl.window_should_close() {
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();

        // Create a scaler for this frame
        let scaler = UIScaler::new(screen_w, screen_h);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(bg_color);

        // // The sidebar marker rectangle
        // d.draw_rectangle(
        //     scaler.scale_w(0.0),  // 4% of width
        //     scaler.scale_h(0.0),  // 6% of height
        //     scaler.scale_w(0.35), // 35% of width
        //     scaler.scale_h(1.00), // 90% of height
        //     Color::RED,
        // );

        // d.draw_rectangle(
        //     scaler.scale_w(0.35), // 4% of width
        //     scaler.scale_h(0.0),  // 6% of height
        //     scaler.scale_w(0.65), // 35% of width
        //     scaler.scale_h(1.00), // 90% of height
        //     Color::BLUE,
        // );

        // // temporary board location
        // d.draw_rectangle(
        //     scaler.scale_w(1.00) - scaler.scale_h(0.925),
        //     scaler.scale_h(0.075),
        //     scaler.scale_h(0.88),
        //     scaler.scale_h(0.88),
        //     Color::GREEN,
        // );

        // The top menu bar marker rectangle
        d.draw_rectangle(
            scaler.scale_w(0.0),
            scaler.scale_h(0.0),
            scaler.scale_w(1.0),
            scaler.scale_h(0.035),
            Color::WHITE,
        );

        // // Example: draw using rect helper (x, y, w, h are percentages)
        // let rect = scaler.rect(0.5, 0.1, 0.3, 0.3);
        // if d.gui_button(rect, "Click Me") {
        //     println!("Button clicked!");
        // }
    }
}
