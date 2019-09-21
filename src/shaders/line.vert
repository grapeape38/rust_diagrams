#version 330 core

layout (location = 0) in vec2 Position;
uniform vec2 point1;

void main() {
    gl_Position = vec4(point1, 0.0, 1.0);
}