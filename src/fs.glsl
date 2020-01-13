in vec4		v_color;
in vec3		v_normal;

out vec3	out_color;

uniform float	time;


mat4 rotationX( in float angle ) {
	return mat4(	1.0,		0,			0,			0,
			 		0, 	cos(angle),	-sin(angle),		0,
					0, 	sin(angle),	 cos(angle),		0,
					0, 			0,			  0, 		1);
}

mat4 rotationY( in float angle ) {
	return mat4(	cos(angle),		0,		sin(angle),	0,
			 				0,		1.0,			 0,	0,
					-sin(angle),	0,		cos(angle),	0,
							0, 		0,				0,	1);
}

mat4 rotationZ( in float angle ) {
	return mat4(	cos(angle),		-sin(angle),	0,	0,
			 		sin(angle),		cos(angle),		0,	0,
							0,				0,		1,	0,
							0,				0,		0,	1);
}

void main() {
 // object color
  vec3 obj_color = vec3(.6, .6, .6);

  // light direction
  mat4 light_rot = rotationX(cos(time)) * rotationY(sin(time));
  vec3 light_dir = vec3((light_rot * vec4(0., -1., -.5, 1.0)).xyz);

  // diffusion factor (hence the k)
  float kd = dot(v_normal, -light_dir);

  out_color = (obj_color * kd // + v_normal
  ) // / 2.0
  ;
}
