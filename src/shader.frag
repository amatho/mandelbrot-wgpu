#version 450

layout(origin_upper_left) in vec4 gl_FragCoord;
layout(location = 0) out vec4 outColor;

uniform Locals {
    double screenWidth;
    double screenHeight;
    double maxIter;
    double pixelDelta;
    double centerRe;
    double centerIm;
};

double pow2(double x) {
    return x * x;
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    double c_re = centerRe + (pixelDelta * (gl_FragCoord.x - double(screenWidth) / 2));
    double c_im = centerIm - (pixelDelta * (gl_FragCoord.y - double(screenHeight) / 2));
    dvec2 c = dvec2(c_re, c_im);
    dvec2 z = c;
    
    float i;
    for(i = 0; i < maxIter; i++) {
        z = dvec2(pow2(z.x) - pow2(z.y), 2 * z.x * z.y) + c;

        if (pow2(z.x) + pow2(z.y) > 4) {
            break;
        }
    }

    if (i == maxIter) {
        outColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        float val = i / float(maxIter);
        outColor = vec4(hsv2rgb(vec3(val, 1.0, 1.0)), 1.0);
    }
}
