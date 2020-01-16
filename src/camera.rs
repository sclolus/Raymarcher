//! The camera component of the View component of the system.

use cgmath::{Vector3, Point3, Rotation, Rotation3, Matrix4, PerspectiveFov, InnerSpace, Rad, Deg, Quaternion};

pub struct Camera {
	position: Point3<f32>,
	dir: Vector3<f32>,
	up: Vector3<f32>,
	aspect: f32,
	fovy: Rad<f32>,
	near_distance: f32,
	far_distance: f32,
}

pub enum Direction {
	Forward,
	Backward,
	Up,
	Down,
	Left,
	Right,
}

impl Camera {
	pub fn with_aspect(&mut self, width: f32, height: f32) -> &mut Self {
		self.aspect = width / height;
		self
	}

	pub fn with_position(&mut self, position: Point3<f32>) -> &mut Self {
		self.position = position;
		self
	}

	// pub fn with_dir(&mut self, dir: Vector3<f32>) -> &mut Self {
	// 	self.dir = dir.normalize();
	// 	self
	// }
		

	#[allow(unused)]
	pub fn with_fovy(&mut self, fovy: Deg<f32>) -> &mut Self {
		self.fovy = fovy.into();
		self
	}
	
	#[allow(unused)]
	pub fn with_frustrum_distances(&mut self, near: f32, far: f32) -> &mut Self {
		self.near_distance = near;
		self.far_distance = far;
		self
	}

	pub fn move_in_direction(&mut self,
							 direction: Direction,
							 distance: f32) -> &mut Self {
		use Direction::*;
		let direction_vector = match direction {
			Forward => {
				self.dir
			},
			Backward => {
				-self.dir
			},
			Up => {
				self.up
			},
			Down => {
				-self.up
			},
			Left => {
				-self.dir.cross(self.up)
			},
			Right => {
				self.dir.cross(self.up)
			},			
		};

		self.position += direction_vector.normalize() * distance;
		self
	}

	pub fn right_vector(&self) -> Vector3<f32> {
		self.dir.cross(self.up)
	}
	
	pub fn up_vector(&self) -> Vector3<f32> {
		self.up
	}

	pub fn forward_vector(&self) -> Vector3<f32> {
		self.dir
	}
	
	#[allow(unused)]
	pub fn rotate_dir<A: Into<Rad<f32>>>(&mut self, axis: Vector3<f32>, angle: A) -> &mut Self {
		let rotation_quarternion = Quaternion::from_axis_angle(axis, angle);
		self.dir = rotation_quarternion.rotate_vector(self.dir);
		self.up = rotation_quarternion.rotate_vector(self.up);
		self
	}

	pub fn view_matrix(&self) -> Matrix4<f32> {
		let view_point = self.position + self.dir;
		
		Matrix4::look_at(self.position, view_point, self.up)
	}

	pub fn projection_matrix(&self) -> Matrix4<f32> {
		Matrix4::from(PerspectiveFov {
			fovy: self.fovy,
			aspect: self.aspect,
			near: self.near_distance,
			far: self.far_distance,
		}.to_perspective())
	}
}

impl Default for Camera {
	fn default() -> Self {
		Self {
			position: Point3::new(0.0, 0.0, 1.0),
			dir: Vector3::new(0.0, 0.0, -1.0),
			up: Vector3::new(0.0, 1.0, 0.0),
			aspect: 0.75,
			fovy: cgmath::Deg(90.).into(),
			near_distance: 0.1,
			far_distance: 1000.,
		}
	}
}
