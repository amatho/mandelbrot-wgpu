#version 450

in vec4 gl_FragCoord;
layout(location = 0) out vec4 outColor;

uniform Locals {
    vec2 screenSize;
    vec2 center;
    float scale;
    int maxIter;
};

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 spectralColor(float iter_frac) {
    iter_frac *= 300.0;

    float r = 0.0;
    float g = 0.0;
    float b = 0.0;
    if ((iter_frac >= 0.0) && (iter_frac < 10.0)) {
        float t = iter_frac / 10.0;
        r = (0.33 * t) - (0.20 * t * t);
    } else if ((iter_frac >= 10.0) && (iter_frac < 75.0)) {
        float t = (iter_frac - 10.0) / 65.0;
        r = 0.14 - (0.13 * t * t);
    } else if ((iter_frac >= 145.0) && (iter_frac < 195.0)) {
        float t = (iter_frac - 145.0) / 50.0;
        r = (1.98 * t) - (t * t);
    } else if ((iter_frac >= 195.0) && (iter_frac < 250.0)) {
        float t = (iter_frac - 195.0) / 55.0;
        r = 0.98 + (0.06 * t) - (0.40 * t * t);
    } else if ((iter_frac >= 250.0) && (iter_frac < 300.0)) {
        float t = (iter_frac - 250.0) / 50.0;
        r = 0.65 - (0.84 * t) + (0.20 * t * t);
    }

    if ((iter_frac >= 15.0) && (iter_frac < 75.0)) {
        float t = (iter_frac - 15.0) / 60.0;
        g = (0.80 * t * t);
    } else if ((iter_frac >= 75.0) && (iter_frac < 190.0)) {
        float t = (iter_frac - 75.0) / 115.0;
        g = 0.8 + (0.76 * t) - (0.80 * t * t);
    } else if ((iter_frac >= 185.0) && (iter_frac < 239.0)) {
        float t = (iter_frac - 185.0) / 54.0;
        g = 0.84 - (0.84 * t);
    }

    if ((iter_frac >= 0.0) && (iter_frac < 75.0)) {
        float t = iter_frac / 75.0;
        b = (2.20 * t) - (1.50 * t * t);
    } else if ((iter_frac >= 75.0) && (iter_frac < 160.0)) {
        float t = (iter_frac - 75.0) / 85.0;
        b = 0.7 - t + (0.30 * t * t);
    }

    return vec3(r, g, b);
}

void main() {
    float aspectRatio = screenSize.x / screenSize.y;
    vec2 coord;
    coord.x = ((gl_FragCoord.x / screenSize.y) - 0.5 * aspectRatio) * scale;
    coord.y = ((gl_FragCoord.y / screenSize.y) - 0.5) * scale;

    vec2 c = vec2(coord.x + center.x, coord.y - center.y);

    vec2 z = c;
    int i;
    for (i = 0; i < maxIter; i++) {
        float x = (z.x * z.x - z.y * z.y) + c.x;
        float y = (z.y * z.x + z.x * z.y) + c.y;

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
        outColor = vec4(spectralColor(val), 1.0);
    }
}
