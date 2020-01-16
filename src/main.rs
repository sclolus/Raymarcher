#![feature(clamp)]

extern crate luminance;
extern crate luminance_glfw;
#[macro_use]
extern crate luminance_derive;

extern crate wavefront_obj;
extern crate cgmath;
extern crate fps_counter;
#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use cgmath::{Point3, Vector3, Vector2, InnerSpace};
use std::time::Instant;
use luminance_glfw::{Surface, GlfwSurface, WindowOpt, WindowDim, CursorMode, WindowEvent, Key};
use luminance::{
	linear::M44,
	pipeline::{PipelineState},
	context::GraphicsContext,
	shader::{
		program::{Program, Uniform},
		stage::{Type, Stage},
	},
	render_state::RenderState,
	tess::{Mode, TessBuilder, TessSliceIndex},
};

use wavefront_obj::obj;
use std::fs;

mod vertex;

mod camera;

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
struct GlVertex {
	#[allow(unused)]
	position: VertexPosition,
	#[vertex(normalized = "true")]
	#[allow(unused)]
	color: VertexRGBA,
	#[allow(unused)]
	normal: VertexNormal,
}

const STEP: f32 = 1.25;
const CAMERA_SENSIBILITY: f32 = 0.07;

fn main() {
	let matches = clap_app! {
		raymarcher =>
			(version: "omega-0.1")
			(author: "Sebastien `sclolus` CLOLUS <sclolus@student.42.fr>")
			(about: "A simple rust rasterizer/raymarcher (Not sure yet)")
			(bin_name: "Raymarcher")
			(after_help: "A simple raymarcher.")
			(@arg object_files: -f --obj-file +required +takes_value ... "An wavefront .obj object file's name to render")
	}.get_matches();
	pretty_env_logger::init_timed();

	let object_files = matches.values_of("object_files").expect("Should have object_files");
	let window_size = WindowDim::Windowed(1080, 720);
	let window_opt = WindowOpt::default().set_cursor_mode(CursorMode::Disabled).set_num_samples(Some(4));
	let mut surface = GlfwSurface::new(window_size, WINDOW_NAME, window_opt).expect("Failed to create new window surface");

	let tess = None;
	let vertex = Stage::new(Type::VertexShader, VERTEX_SHADER_SOURCE).expect("Vertex shader building failed");
	let geometry = None;
	let fragment = Stage::new(Type::FragmentShader, FRAGMENT_SHADER_SOURCE).expect("Fragment shader building failed");
	
	let shader: Program<VertexSemantics, (), FragmentShaderUniform> = Program::from_stages(tess, &vertex, geometry, &fragment).unwrap().ignore_warnings();

	let object_files = object_files
		.flat_map(|f: &str| fs::read_to_string(f).map_err(|_| panic!("Could not read {} object file", f)))
		.collect::<Vec<String>>();
	
	let obj_sets = object_files.iter().map(|file| obj::parse(file).or_else(|parse_error| {
		warn!("Failed to parse .obj file({}): {}", parse_error.line_number, parse_error.message);
		Err(parse_error)
	}).unwrap()).collect::<Vec<obj::ObjSet>>();

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

			GlVertex {
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
					e => {
						error!("{:?} shape is not supported", e);
						panic!()
					}
				}
			}).collect::<Vec<[GlVertex; 3]>>()
	};

	
	let mut object_vertices = Vec::new();
	for object in obj_sets.iter().flat_map(|set| set.objects.iter()) {
		let new_vertices = object_to_vertices(object);
		info!("New model: {} has {} vertices", object.name, object.vertices.len());
		object_vertices.extend(new_vertices);
	}

	let object_tesses = object_vertices.into_iter()
		.map(|vertices|
			 TessBuilder::new(&mut surface)
			 .add_vertices(vertices)
			 .set_mode(Mode::Triangle)
			 .build().unwrap()
		).collect::<Vec<luminance::tess::Tess>>();

	let mut camera = camera::Camera::default();
	camera.with_aspect(surface.width() as f32, surface.height() as f32);
		// .with_position(Point3::new(5., 5., 5.))
		// .with_dir(Vector3::new(-5., -5., -5.));
	
	let mut fps_counter = fps_counter::FPSCounter::new();
	let start = Instant::now();
	let mut old_cursor_pos = Vector2::new(surface.width() as f32 / 2., surface.height() as f32 / 2.);
	
	info!("Rendering start");
	'app: loop {
		for event in surface.poll_events() {
			if let WindowEvent::Close | WindowEvent::Key(Key::Escape, _, _, _) = event {
				info!("Close event received, closing window...");
				break 'app;
			}

			if let WindowEvent::Key(Key::Up, _, _, _) = event {
				camera.move_in_direction(camera::Direction::Forward, STEP);
			} else if let WindowEvent::Key(Key::Left, _, _, _) = event {
				camera.move_in_direction(camera::Direction::Left, STEP);
			} else if let WindowEvent::Key(Key::Right, _, _, _) = event {
				camera.move_in_direction(camera::Direction::Right, STEP);
			} else if let WindowEvent::Key(Key::Down, _, _, _) = event {
				camera.move_in_direction(camera::Direction::Backward, STEP);
			} else if let WindowEvent::CursorPos(x, y) = event {
				let pos = Vector2::new(x as f32, y as f32);
				let delta_pos = (pos - old_cursor_pos).normalize() * CAMERA_SENSIBILITY;
				let up = camera.up_vector();
				let right = camera.right_vector();
				
				camera.rotate_dir(right, cgmath::Rad(delta_pos.y))
					.rotate_dir(up, cgmath::Rad(-delta_pos.x));
				old_cursor_pos = pos;
			}
		}
		
		let now = Instant::now();
		let time = now.duration_since(start).as_secs_f32();
		let buffer = surface.back_buffer().expect("Failed to get back buffer");
		let mut builder = surface.pipeline_builder();

		let pipeline_state = PipelineState::new()
			.set_clear_color([1.0; 4])
			.enable_clear_color(true)
			.enable_clear_depth(true);

		builder.pipeline(&buffer, &pipeline_state, |_pipeline, mut shading_gate|  {
			shading_gate.shade(&shader, |p_interface, mut render_gate| {
				let view_matrix = camera.view_matrix();
				let projection_matrix = camera.projection_matrix();
				p_interface.time.update(time.into());
				p_interface.view.update(view_matrix.into());
				p_interface.projection.update(projection_matrix.into());
				let render_state = RenderState::default();

				render_gate.render(&render_state, |mut tess_gate| {
					for tess in object_tesses.iter() {
						tess_gate.render(tess.slice(..));
					}
				});
			})
		});
		surface.swap_buffers();
		info!("Rendering at {} fps", fps_counter.tick());
	}
}
