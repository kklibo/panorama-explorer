uniform vec4 color;

in vec3 pos;

layout (location = 0) out vec4 outColor;

void main()
{
    outColor = color;
}