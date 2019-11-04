#version 330 core
layout(location = 0) out vec4 shapeColor;

in vec3 frag_color;

void main()
{
    shapeColor = vec4(frag_color, 1.0);
}