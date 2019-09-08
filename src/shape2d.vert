#version 330 core

layout (location = 0) in vec2 Position;
uniform mat3 scale;
uniform mat3 rotation;
uniform mat3 translate;

void main()
{
    gl_Position = vec4(translate * rotation * scale * vec3(Position, 1.0), 1.0);
}