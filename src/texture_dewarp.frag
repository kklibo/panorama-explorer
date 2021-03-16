
uniform float strength;
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    //float strength = 0.5;
    float zoom = 1.0;

    float aspect_x_to_y = 3.0/2.0;

    float image_center_x = 0.5;
    float image_center_y = 0.5;

    float newX = uvs.x - image_center_x;
    float newY = uvs.y - image_center_y;

    float X_asp = newX;
    float Y_asp = newY / aspect_x_to_y;
    float distance = sqrt(X_asp*X_asp + Y_asp*Y_asp);

    float r = distance * strength;

    float theta;
    if (r == 0.0) {theta = 1.0;}
    else        {theta = atan(r) / r ;}

    float sourceX = image_center_x + theta * newX * zoom;
    float sourceY = image_center_y + theta * newY * zoom;

    outColor = texture(tex, vec2(sourceX, 1.0 - sourceY));

    //debug rings
    if (distance > 0.2 && distance < 0.21) { outColor.xyz = vec3(0,0,0); }
    if (distance > 0.49 && distance < 0.5) { outColor.xyz = vec3(0,1,1); }

    outColor.a = 0.5;
}