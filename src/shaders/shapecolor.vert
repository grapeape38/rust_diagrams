#version 330 core

layout (location = 0) in vec2 position;
layout (location = 1) in vec3 color;
uniform mat4 model;
uniform mat4 projection;

out vec3 frag_color;

void main()
{
    gl_Position = vec4(projection * model * vec4(position.x, position.y, 0.0, 1.0));
    frag_color = vec3(color.r, color.g, color.b);
}