#version 330 core

layout (location = 0) in vec2 Position;
uniform mat3 transform;

void main()
{
    gl_Position = vec4(transform * vec3(Position, 1.0), 1.0);
}