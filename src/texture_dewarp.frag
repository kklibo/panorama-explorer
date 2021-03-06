
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    float strength = 2.0;
    float zoom = 1.0;

    float aspect_x_to_y = 4.0/3.0;


    float imageWidth = 1;
    float imageHeight = 1;

    float halfWidth = 0.5; //aspect?
    float halfHeight = 0.5;


    if (strength == 0) { strength = 0.00001; }

    float correctionRadius = sqrt(imageWidth*imageWidth + imageHeight*imageHeight) / strength;


    float newX = uvs.x - halfWidth;
    float newY = uvs.y - halfHeight;

    float distance = sqrt(newX*newX + newY*newY);
    float r = distance / correctionRadius;

    float theta;
    if (r == 0) {theta = 1;}
    else        {theta = atan(r) / r ;}

    float sourceX = halfWidth + theta * newX * zoom;
    float sourceY = halfHeight + theta * newY * zoom;

    //set color of pixel (x, y) to color of source image pixel at (sourceX, sourceY)

    outColor = texture(tex, vec2(sourceX, 1.0 - sourceY));



    //outColor = texture(tex, vec2(uvs.x, 1.0 - uvs.y));
    outColor.a = 0.5;
}