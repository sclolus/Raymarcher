in vec4		v_color;
in vec3		v_normal;

out vec3	out_color;

uniform float	time;

void main() {
 // object color
  vec3 obj_color = vec3(.6, .6, .6);

  // light direction
  vec3 light_dir = vec3(0., -1., -.5);

  // diffusion factor (hence the k)
  float kd = dot(v_normal, -light_dir);

  out_color = (obj_color * kd // + v_normal
  ) // / 2.0
  ;
}
