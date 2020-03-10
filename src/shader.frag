#version 450

in vec4 gl_FragCoord;
layout(location = 0) out vec4 outColor;

uniform Locals {
    double screenWidth;
    double screenHeight;
    double maxIter;
    double scale;
    double centerRe;
    double centerIm;
};

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    dvec2 c;
    c.x = (gl_FragCoord.x - screenWidth / 2) * scale + centerRe;
    c.y = (gl_FragCoord.y - screenHeight / 2) * scale - centerIm;

    dvec2 z = c;
    int i;
    for (i = 0; i < maxIter; i++) {
        double x = (z.x * z.x - z.y * z.y) + c.x;
        double y = (z.y * z.x + z.x * z.y) + c.y;

        if ((x * x + y * y) > 4.0) {
            break;
        }

        z.x = x;
        z.y = y;
    }

    if (i == maxIter) {
        outColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float val = i / float(maxIter);
        outColor = vec4(hsv2rgb(vec3(val, 1.0, 1.0)), 1.0);
    }
}
