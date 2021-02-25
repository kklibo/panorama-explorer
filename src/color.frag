uniform vec4 color;

in vec3 pos;
in vec3 nor;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    outColor = color;//vec4(1.0, 0.5, 0.5, 1.0);
}