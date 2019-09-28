#version 330 core

layout (points) in;
layout (line_strip, max_vertices = 2) out;

uniform vec2 point1;
uniform vec2 point2;
uniform mat4 projection;

void main()
{
    gl_Position = projection *  vec4(point1, 0.0, 1.0);
    EmitVertex();
    gl_Position = projection * vec4(point2, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
}