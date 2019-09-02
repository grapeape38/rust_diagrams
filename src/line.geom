#version 330 core

layout (points) in;
layout (line_strip, max_vertices = 2) out;

uniform vec2 point2;

void main()
{
    gl_Position = gl_in[0].gl_Position;
    EmitVertex();
    gl_Position = vec4(point2, 0.0, 1.0);
    EmitVertex();
    EndPrimitive();
}