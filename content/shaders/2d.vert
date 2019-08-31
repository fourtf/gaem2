#version 400

layout(location = 0) in vec3 vertPos;
layout(location = 1) in vec2 texCoord;

uniform mat4 viewMatrix, projMatrix;

out vec2 tex_coord;

void main() {
  gl_Position = projMatrix * viewMatrix * vec4(vertPos, 1.0);
  tex_coord = gl_Position.xy;
}