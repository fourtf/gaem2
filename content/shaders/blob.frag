#version 400

layout(location = 0) out vec4 frag_color;

// varying vec2 texCoord;
uniform sampler2D texture1;
// in vec2 tex_coord;

void main() {
  // frag_color = vec4(tex_coord.x, tex_coord.y, 1.0, 1.0);

  vec4 pixel = gl_FrontColor;

  if (pixel.g > 0.5) {
    frag_color = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    frag_color = pixel;
  }
}
