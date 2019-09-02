#version 330 core
out vec4 FragColor;

uniform vec4 color;

void main()
{
    FragColor = color;
    //FragColor = vec4(0.0, 1.0, 0.0, 1.0); 
}