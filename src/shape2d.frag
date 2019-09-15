#version 330 core
layout(location = 0) out vec4 shapeColor;

uniform vec4 color;

void main()
{
    shapeColor = color;
}