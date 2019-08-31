#version 400

layout(location = 0) out vec4 frag_color;

void main() {
  frag_color.x =
      abs(sin(gl_FragCoord.x / 100.0) / 7.0 + tan(gl_FragCoord.z / 11.0)) * 0.3;
  frag_color.y =
      abs(sin(gl_FragCoord.y / 130.0) + sin(gl_FragCoord.x / 1000.0)) * 0.8;
  frag_color.z = 0.7;
  frag_color.w = 1.0;
}