#![feature(clamp)]

extern crate piston;
extern crate piston_window;

use piston_window::*;
use piston_window::color::hex;
use grid::Grid;
use line::Line;
use std::f64;

fn main() {
    println!("Hello, world!");

	let opengl_api_version = OpenGL::V3_2;
	let mut window: PistonWindow = WindowSettings::new(
        "piston: draw_state",
        [1080, 720]
    )
		.title("RustMarcher".to_owned())
		.fullscreen(true)
        .exit_on_esc(true)
		.graphics_api(opengl_api_version)
		.resizable(true)
		.vsync(true)
        .samples(4)
        .build()
        .unwrap();
	window.set_lazy(true);
	
	let mut wsize = window.size();
	let make_grid = |scale| Grid {
		cols: (wsize.width / scale) as u32 + 1,
		rows: (wsize.height / scale) as u32 + 1,
		units: scale,
	};
	let mut grid = make_grid(10.0);
	
	let grid_lines = Line::new_round(hex("00ffff"), 2.0);
	let grid_draw_state = DrawState::new_alpha();
	
	while let Some(event) = window.next() {
		println!("Event {:?} dispatched", event);
		window.draw_2d(&event, |context, graphics, _| {
			clear([1.0; 4], graphics);
			// rectangle(hex("ff0000"), // red
            //           [0.0, 0.0, 100.0, 100.0],
            //           context.transform,
            //           graphics);
			// polygon(hex("ff0000"), &[[200., 200.], [300., 600.], [800., 800.]], context.transform, graphics);
			grid.draw(&grid_lines, &grid_draw_state, context.transform, graphics);
		});
		
		if let Some(button) = event.press_args() {
			if button == Button::Keyboard(Key::Minus) {
				grid = make_grid(grid.units + 1.);
			} else if button == Button::Keyboard(Key::Equals) {
				grid = make_grid((grid.units - 1.).clamp(0.1, f64::INFINITY))
			}
		}
	}
}
