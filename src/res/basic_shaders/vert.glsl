#version 330

in vec2 aPos;
out vec2 iUv;

void main() {
    gl_Position = vec4(aPos, 0.0, 1.0);
    iUv = aPos * 0.5 + 0.5;
}
