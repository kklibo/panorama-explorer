uniform sampler2D colorMap;
uniform sampler2D depthMap;
in vec2 uv;
layout (location = 0) out vec4 color;
void main()
{
    color = texture(colorMap, uv);
    gl_FragDepth = texture(depthMap, uv).r;
}