in vec3 position;
in vec4 color;
in vec3 normal;

uniform mat4 view;
uniform mat4 projection;

out vec4 v_color;
out vec3 v_normal;

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
	float anglex = cos(time);
	float angley = sin(time);
	mat4 rot = rotationX(anglex) * rotationY(angley) * rotationZ(anglex);

	v_color = color;
	v_normal = (vec4(normal, 1.0)).xzy;
	gl_Position = projection * view // * rot
	
	* vec4(position, 1.);
}
