#![feature(clamp)]

extern crate piston;
extern crate piston_window;
extern crate luminance;
extern crate luminance_glfw;
#[macro_use]
extern crate luminance_derive;

extern crate wavefront_obj;
extern crate cgmath;

use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3, PerspectiveFov};
use piston_window::*;
use piston_window::color::hex;
use grid::Grid;
use line::Line;
use std::f64;
use std::time::Instant;
use std::process;
use luminance_glfw::{Surface, GlfwSurface, WindowOpt, WindowDim, CursorMode, WindowEvent, Key};
use luminance::{
	linear::M44,
	pipeline::{PipelineState, self},
	context::GraphicsContext,
	shader::{
		program::{Program, Uniform},
		stage::{Type, Stage},
	},
	render_state::RenderState,
	tess::{Mode, TessBuilder, TessSlice, TessSliceIndex},
	vertex::{Vertex as VertexTrait, Semantics},
};

use wavefront_obj::obj;
use wavefront_obj::ParseError;
use std::fs;

const WINDOW_NAME: &str = "Raymarcher";
const VERTEX_SHADER_SOURCE: &str = include_str!("vs.glsl");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("fs.glsl");

#[derive(Clone, Copy, Debug, Semantics)]
pub enum VertexSemantics {
	#[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
	Position,
	#[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexRGBA")]
	Color,
	#[sem(name = "normal", repr = "[f32; 3]", wrapper = "VertexNormal")]
	Normal,
}

#[derive(Debug, UniformInterface)]
pub struct FragmentShaderUniform {
	#[uniform(unbound)]
	time: Uniform<f32>,
	#[uniform(unbound)]
	view: Uniform<M44>,
	#[uniform(unbound)]
	projection: Uniform<M44>
}

#[derive(Vertex)]
#[vertex(sem = "VertexSemantics")]
struct Vertex {
	position: VertexPosition,
	#[vertex(normalized = "true")]
	color: VertexRGBA,
	normal: VertexNormal,
}

const STEP: f32 = 0.5;
fn main() {
	let vertices_filename = match std::env::args().nth(1) {
		Some(filename) => filename,
		None => {
			eprintln!("No input .obj file was provided");
			std::process::exit(1);
		},
	};
	let window_size = WindowDim::Windowed(1080, 720);
	let window_opt = WindowOpt::default().set_cursor_mode(CursorMode::Visible).set_num_samples(Some(4));

	let mut surface = GlfwSurface::new(window_size, WINDOW_NAME, window_opt).expect("Failed to create new window surface");

	let tess = None;
	let vertex = Stage::new(Type::VertexShader, VERTEX_SHADER_SOURCE).expect("Vertex shader building failed");
	let geometry = None;
	let fragment = Stage::new(Type::FragmentShader, FRAGMENT_SHADER_SOURCE).expect("Fragment shader building failed");
	
	let shader: Program<VertexSemantics, (), FragmentShaderUniform> = Program::from_stages(tess, &vertex, geometry, &fragment).unwrap().ignore_warnings();

	let vertices_file = fs::read_to_string(vertices_filename).expect("Failed to read vertices file");
	
	let obj_set = obj::parse(vertices_file).or_else(|parse_error| {
		println!("Failed to parse .obj file({}): {}", parse_error.line_number, parse_error.message);
		Err(parse_error)
	}).unwrap();

	let object_to_vertices = |object: &obj::Object| {
		let wavefront_vertex_to_vertex = |object: &obj::Object, indices: (obj::VTNIndex, obj::VTNIndex, obj::VTNIndex)| {
			let vertex = object.vertices[(indices.0).0];
			let v2 = object.vertices[(indices.1).0];
			let v3 = object.vertices[(indices.2).0];
			let mv = |v: obj::Vertex| Vector3::new(v.x, v.y, v.z);
			
			let normal;
			if let Some(ni) = (indices.0).2 {
				normal = object.normals[ni];
			} else {
				let u = mv(v2) - mv(vertex);
				let v = mv(v3) - mv(vertex);

				let n = -u.cross(v);
				normal = obj::Vertex {
					x: n.x,
					y: n.y,
					z: n.z,
				};
			}

			Vertex {
				position: VertexPosition::new([vertex.x as f32, vertex.y as f32, vertex.z as f32]),
				color: VertexRGBA::new([1.0, 0.0, 0.0, 0.0]),
				normal: VertexNormal::new([normal.x as f32, normal.y as f32, normal.z as f32]),
			}
		};
		
		object.geometry.iter()
			.flat_map(|geometry| geometry.shapes.iter())
			.map(|shape| {
				match shape.primitive {
					obj::Primitive::Triangle(v0, v1, v2) => {
						[wavefront_vertex_to_vertex(object, (v0, v1, v2)),
						 wavefront_vertex_to_vertex(object, (v1, v0, v2)),
						 wavefront_vertex_to_vertex(object, (v2, v1, v0))]
					},
					_ => panic!("{:?} shape is not supported"),
				}
			}).collect::<Vec<[Vertex; 3]>>()
	};


	
	let mut object_vertices = Vec::new();
	for object in obj_set.objects.iter() {
		let new_vertices = object_to_vertices(object);
		println!("New model: {} has {} triangles", object.name, object_vertices.len());
		object_vertices.extend(new_vertices);
	}


	let object_tesses = object_vertices.into_iter()
		.map(|vertices|
			 TessBuilder::new(&mut surface)
			 .add_vertices(vertices)
			 .set_mode(Mode::Triangle)
			 .build().unwrap()
		).collect::<Vec<luminance::tess::Tess>>();

	let mut eye_pos = Point3::new(0.0, 0.0, 1.0);
	let dir = Vector3::new(0.0, 0.0, -1.0);
	let up_dir = Vector3::new(0.0, 1.0, 0.0);
	let aspect = surface.width() as f32 / surface.height() as f32;
	let make_camera_matrices = |eye_pos, dir, up_dir, aspect| {
		let eye_center = eye_pos + dir;
		let view_matrix = Matrix4::look_at(eye_pos, eye_center, up_dir);
		let projection_matrix = Matrix4::from(PerspectiveFov {
			fovy: cgmath::Deg(90.).into(),
			aspect,
			near: 0.1,
			far: 1000.,
		}.to_perspective());
		(view_matrix, projection_matrix)
	};
	let (mut view_matrix, mut projection_matrix) = make_camera_matrices(eye_pos, dir, up_dir, aspect);
	let start = Instant::now();

	'app: loop {
		for event in surface.poll_events() {
			if let WindowEvent::Close | WindowEvent::Key(Key::Escape, _, _, _) = event {
				println!("Close event received, closing window...");
				break 'app;
			}
			if let WindowEvent::Key(Key::Up, _, _, _) = event {
				eye_pos += dir * STEP;
				let new_matrices = make_camera_matrices(eye_pos, dir, up_dir, aspect);
				view_matrix = new_matrices.0;
				projection_matrix = new_matrices.1;
			}
			if let WindowEvent::Key(Key::Down, _, _, _) = event {
				eye_pos -= dir * STEP;
				let new_matrices = make_camera_matrices(eye_pos, dir, up_dir, aspect);
				view_matrix = new_matrices.0;
				projection_matrix = new_matrices.1;
			}
			
			println!("{:?}", event);

		}
		let now = Instant::now();
		let time = now.duration_since(start).as_secs_f32();
		let buffer = surface.back_buffer().expect("Failed to get back buffer");
		let mut builder = surface.pipeline_builder();

		let pipeline_state = PipelineState::new().set_clear_color([1.0; 4]).enable_clear_color(true).enable_clear_depth(true);
		

		builder.pipeline(&buffer, &pipeline_state, |pipeline, mut shading_gate|  {
			shading_gate.shade(&shader, |p_interface, mut render_gate| {
				p_interface.time.update(time.into());
				p_interface.view.update(view_matrix.into());
				p_interface.projection.update(projection_matrix.into());
				let render_state = RenderState::default();

				render_gate.render(&render_state, |mut tess_gate| {
					for tess in object_tesses.iter() {
						tess_gate.render(tess.slice(..));
					}
					// tess_gate.render(triangle.slice(..));
				});
			})
		});
		surface.swap_buffers();
		println!("Took {} secs to render", Instant::now().duration_since(now).as_secs_f32());
	}
}
// fn main() {
//     println!("Hello, world!");

// 	let opengl_api_version = OpenGL::V3_2;
// 	let mut window: PistonWindow = WindowSettings::new(
//         "piston: draw_state",
//         [1080, 720]
//     )
// 		.title("RustMarcher".to_owned())
// 		.fullscreen(true)
//         .exit_on_esc(true)
// 		.graphics_api(opengl_api_version)
// 		.resizable(true)
// 		.vsync(true)
//         .samples(4)
//         .build()
//         .unwrap();
// 	window.set_lazy(true);
	
// 	let mut wsize = window.size();
// 	let make_grid = |scale| Grid {
// 		cols: (wsize.width / scale) as u32 + 1,
// 		rows: (wsize.height / scale) as u32 + 1,
// 		units: scale,
// 	};
// 	let mut grid = make_grid(10.0);
	
// 	let grid_lines = Line::new_round(hex("00ffff"), 2.0);
// 	let grid_draw_state = DrawState::new_alpha();
	
// 	while let Some(event) = window.next() {
// 		println!("Event {:?} dispatched", event);
// 		window.draw_2d(&event, |context, graphics, _| {
// 			clear([1.0; 4], graphics);
// 			// rectangle(hex("ff0000"), // red
//             //           [0.0, 0.0, 100.0, 100.0],
//             //           context.transform,
//             //           graphics);
// 			// polygon(hex("ff0000"), &[[200., 200.], [300., 600.], [800., 800.]], context.transform, graphics);
// 			grid.draw(&grid_lines, &grid_draw_state, context.transform, graphics);
// 		});
		
// 		if let Some(button) = event.press_args() {
// 			if button == Button::Keyboard(Key::Minus) {
// 				grid = make_grid(grid.units + 1.);
// 			} else if button == Button::Keyboard(Key::Equals) {
// 				grid = make_grid((grid.units - 1.).clamp(0.1, f64::INFINITY))
// 			}
// 		}
// 	}
// }
