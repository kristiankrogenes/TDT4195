#version 430 core

in layout(location=1) vec4 newColor;

out vec4 color;

void main()
{
    color = newColor;
}