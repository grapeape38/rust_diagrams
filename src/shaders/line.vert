#version 330 core

layout (location = 0) in vec4 Position;

void main() {
    gl_Position = Position; 
}