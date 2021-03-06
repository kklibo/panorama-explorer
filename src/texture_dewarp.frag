
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    float strength = 0.5;
    float zoom = 1.0;

    float aspect_x_to_y = 3.0/2.0;


    float imageWidth = 1;
    float imageHeight = 1;// / aspect_x_to_y;

    float halfWidth = imageWidth * 0.5;
    float halfHeight = imageHeight * 0.5;


    if (strength == 0) { strength = 0.00001; }

    //float correctionRadius = sqrt(imageWidth*imageWidth + imageHeight*imageHeight) / strength;
    //float correctionRadius = sqrt((1/aspect_x_to_y)*(1/aspect_x_to_y) + 1) / strength;


    float newX = uvs.x - halfWidth;
    float newY = uvs.y - halfHeight;

    float X_asp = newX;
    float Y_asp = newY / aspect_x_to_y;

    //float distance = sqrt(newX*newX + newY*newY);
    float distance = sqrt(X_asp*X_asp + Y_asp*Y_asp);
    //float r = distance / correctionRadius;
    float r = distance * strength;

    float theta;
    if (r == 0) {theta = 1;}
    else        {theta = atan(r) / r ;}

    float sourceX = halfWidth + theta * newX * zoom;
    //float sourceY = halfHeight + theta * newY * zoom;
    float sourceY = halfHeight + (theta * newY * zoom );// / aspect_x_to_y);
    //float sourceY = halfHeight + theta * Y_asp * zoom;

    //set color of pixel (x, y) to color of source image pixel at (sourceX, sourceY)

    outColor = texture(tex, vec2(sourceX, 1.0 - sourceY));

    if (r > 0.2 && r < 0.21) { outColor.xyz = vec3(0,0,0); }

    if (r > 0.99 && r < 1) { outColor.xyz = vec3(0,1,1); }

    //outColor = texture(tex, vec2(uvs.x, 1.0 - uvs.y));
    outColor.a = 0.5;
}