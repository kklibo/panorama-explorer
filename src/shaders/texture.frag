
uniform sampler2D tex;
uniform float out_alpha;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    outColor = texture(tex, vec2(uvs.x, 1.0 - uvs.y));
    outColor.a = out_alpha;
}