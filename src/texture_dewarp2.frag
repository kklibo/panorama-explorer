
uniform float strength;
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    float aspect_x_to_y = 3.0/2.0;

    float image_center_x = 0.5;
    float image_center_y = 0.5;

    float newX = uvs.x - image_center_x;
    float newY = uvs.y - image_center_y;

    float X_asp = newX;
    float Y_asp = newY / aspect_x_to_y;
    float distance = sqrt(X_asp*X_asp + Y_asp*Y_asp);

    float rU = distance;
    float a = 0.0019098468424889991;
    float b = -0.0028266879132016103;
    float c = 0.009532148272374459;


    //todo: rename, handle div by 0

    //temp radius compensation

    rU *= 2;
    rU *= aspect_x_to_y;


    float rD = a * pow(rU,4) + b * pow(rU,3) + c * pow(rU,2) + (1 - a - b - c) * rU;

    float ratio = rD / rU;


    float xD = image_center_x + newX * ratio;
    float yD = image_center_y + newY * ratio;

    outColor = texture(tex, vec2(xD, 1.0 - yD));

    //debug rings
    if (rU > 0.4 && rU < 0.41) { outColor.xyz = vec3(1,0,0); }
    if (rU > 0.99 && rU < 1) { outColor.xyz = vec3(0,1,1); }

    outColor.a = 0.5;
}

/*

focal="200" a="0.0019098468424889991" b="-0.0028266879132016103" c="0.009532148272374459"

Rd = a * Ru^4 + b * Ru^3 + c * Ru^2 + (1 - a - b - c) * Ru
"Ru is the radius of the undistorted pixel, Rd is the radius of the distorted pixel"

"the largest circle that completely fits into an image is said to have radius=1.0"
*/