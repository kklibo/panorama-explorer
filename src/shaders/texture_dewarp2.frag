
uniform sampler2D tex;

in vec3 pos;
in vec2 uvs;

layout (location = 0) out vec4 outColor;

void main()
{
    float aspect_x_to_y = 920.0/614.0;

    vec2 image_center = vec2(0.5, 0.5);

    //image coordinates, relative to image center
    vec2 image_coords = uvs - image_center;

    //aspect ratio correction for radius calculation
    vec2 asp_coords = image_coords;
         asp_coords.x *= aspect_x_to_y;

    //radius (from image center), Undistorted
    float rU = distance(vec2(0.0), asp_coords);

    //ptlens/panotools-style polynomial distortion parameters
    float a = 0.0019098468424889991;
    float b = -0.0028266879132016103;
    float c = 0.009532148272374459;

    //double the radius:
    // ptlens/panotools-style algorithm expects texture range: |[-1,1]| = 2
    // glsl uses |[0,1]| = 1
    rU *= 2.0;


    //radius (from image center), Distorted
    float rD =
    //ptlens/panotools-style polynomial distortion algorithm:
    a * pow(rU,4.0) + b * pow(rU,3.0) + c * pow(rU,2.0) + (1.0 - a - b - c) * rU;


    float ratio;
    if (rU != 0.0) { ratio = rD / rU; }
    else           { ratio = 0.0;     }

    //distorted coordinates: apply new radius from center
    vec2 distorted = image_center + image_coords * ratio;

    //sample texture (flip y-coord)
    outColor = texture(tex, vec2(distorted.x, 1.0 - distorted.y));

    //debug rings
//    if (rU > 0.49 && rU < 0.5) { outColor.xyz = vec3(1,0,0); }
//    if (rU > 0.99 && rU < 1.0) { outColor.xyz = vec3(0,1,1); }

    outColor.a = 0.5;

    //don't render texture samples from outside the image borders
    if (distorted.x < 0.0 || distorted.x > 1.0) { outColor.xyzw = vec4(0,0,0,0); }
    if (distorted.y < 0.0 || distorted.y > 1.0) { outColor.xyzw = vec4(0,0,0,0); }

}

/*

focal="200" a="0.0019098468424889991" b="-0.0028266879132016103" c="0.009532148272374459"

Rd = a * Ru^4 + b * Ru^3 + c * Ru^2 + (1 - a - b - c) * Ru
"Ru is the radius of the undistorted pixel, Rd is the radius of the distorted pixel"

"the largest circle that completely fits into an image is said to have radius=1.0"
*/