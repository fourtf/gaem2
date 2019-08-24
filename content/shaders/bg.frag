void main() {
  gl_FragColor.x =
      abs(sin(gl_FragCoord.x / 100.0) / 7.0 + tan(gl_FragCoord.z / 11.0)) * 0.3;
  gl_FragColor.y =
      abs(sin(gl_FragCoord.y / 130.0) + sin(gl_FragCoord.x / 1000.0)) * 0.8;
  gl_FragColor.z = 0.7;
  gl_FragColor.w = 1.0;
}