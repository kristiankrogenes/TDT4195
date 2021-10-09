#version 430 core

in layout(location=1) vec4 newColor;
in layout(location=2) vec3 newNormal;

out vec4 color;

void main()
{
    // color = vec4(newNormal, 1.0);

    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    vec3 newRGB = newColor.rgb * max(0, dot(newNormal, -lightDirection));
    color = vec4(newRGB, newColor.a);
}