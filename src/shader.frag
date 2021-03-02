#version 450

precision highp float;

in vec4 gl_FragCoord;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform Locals {
    vec2 screenSize;
    vec2 center;
    float scale;
    uint maxIter;
};

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
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
        float hue = i / float(maxIter);
        float saturation = 1.0;
        float value = 0.7;
        outColor = vec4(hsv2rgb(vec3(mod(hue + 0.75, 1.0), saturation, value)), 1.0);
    }
}
