#version 430 core

in layout(location=1) vec4 newColor;
in layout(location=2) vec3 newNormal;

out layout(location=1) vec4 color;

void main()
{
    // vec3 a = normalize(vec3(0.8, -0.5, 0.6));
    // vec3 b = max(vec3(0.0,0.0,0.0), newNormal * (-a));
    // vec4 c = vec4(b, 1.0);
    color = vec4(newNormal, 1.0);
}