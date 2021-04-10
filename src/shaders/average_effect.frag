uniform sampler2D colorMap;
uniform sampler2D depthMap;
in vec2 uv;
layout (location = 0) out vec4 color;
void main()
{
    vec4 source = texture(colorMap, uv);
    if (source.a == 0) {
        color = vec4(0.,0.,0.,1.);
    }
    else {
        color.rgb = source.rgb / source.a;
        color.a = 1.;
    }
    gl_FragDepth = texture(depthMap, uv).r;
}