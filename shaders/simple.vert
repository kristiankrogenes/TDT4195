#version 430 core

in layout(location=0) vec3 position;
in layout(location=1) vec4 color;
in layout(location=2) vec3 normal;

out layout(location=1) vec4 outColor;
out layout(location=2) vec3 outNormal;

uniform layout(location=3) mat4 inMVPTranslation;
uniform layout(location=4) mat4 inModelTranslation;

vec4 positionXYZW = vec4(position.x, position.y, position.z, 1.0);

void main()
{
    gl_Position =  inMVPTranslation * positionXYZW;

    outColor = color;
    outNormal = normalize(mat3(inModelTranslation) * normal);
    // outNormal = normal;
}