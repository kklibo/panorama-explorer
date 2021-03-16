
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    float aspect_x_to_y = 920.0/614.0;

    float image_center_x = 0.5;
    float image_center_y = 0.5;

    //image coordinates, relative to image center
    float image_x = uvs.x - image_center_x;
    float image_y = uvs.y - image_center_y;

    //aspect ratio correction for radius calculation
    float x_asp = image_x * aspect_x_to_y;
    float y_asp = image_y;

    //radius (from image center), Undistorted
    float rU = sqrt(x_asp*x_asp + y_asp*y_asp);

    //ptlens/panotools-style polynomial distortion parameters
    float a = 0.0019098468424889991;
    float b = -0.0028266879132016103;
    float c = 0.009532148272374459;

    //double the radius:
    // ptlens/panotools-style algorithm expects texture range: |[-1,1]| = 2
    // glsl uses |[0,1]| = 1
    rU *= 2;


    //radius (from image center), Distorted
    float rD =
    //ptlens/panotools-style polynomial distortion algorithm:
    a * pow(rU,4) + b * pow(rU,3) + c * pow(rU,2) + (1 - a - b - c) * rU;


    float ratio;
    if (rU != 0) { ratio = rD / rU; }
    else         { ratio = 0;       }


    float xD = image_center_x + image_x * ratio;
    float yD = image_center_y + image_y * ratio;

    outColor = texture(tex, vec2(xD, 1.0 - yD));

    //debug rings
    if (rU > 0.49 && rU < 0.5) { outColor.xyz = vec3(1,0,0); }
    if (rU > 0.99 && rU < 1)   { outColor.xyz = vec3(0,1,1); }

    outColor.a = 0.5;

    //don't render texture samples from outside the image borders
    if (xD < 0 || xD > 1) { outColor.xyzw = vec4(0,0,0,0); }
    if (yD < 0 || yD > 1) { outColor.xyzw = vec4(0,0,0,0); }

}

/*

focal="200" a="0.0019098468424889991" b="-0.0028266879132016103" c="0.009532148272374459"

Rd = a * Ru^4 + b * Ru^3 + c * Ru^2 + (1 - a - b - c) * Ru
"Ru is the radius of the undistorted pixel, Rd is the radius of the distorted pixel"

"the largest circle that completely fits into an image is said to have radius=1.0"
*/