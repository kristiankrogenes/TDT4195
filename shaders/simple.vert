#version 430 core

in layout(location=0) vec3 position;
in layout(location=1) vec4 color;

out layout(location=1) vec4 outColor;

// uniform layout(location=2) float elapsedNum;
uniform layout(location=3) mat4 inMTX;

vec4 positionXYZW = vec4(position.x, position.y, position.z, 1.0f);

// TASK 3
// mat4x4 matrix = mat4(
//     1.0, 0.0, 0.0, 0.0, 
//     elapsedNum, 1.0, 0.0, 0.0, 
//     0.0, 0.0, 1.0, 0.0, 
//     0.0, 0.0, 0.0, 1.0
// );

void main()
{
    gl_Position =  inMTX * positionXYZW;
    outColor = color;
}