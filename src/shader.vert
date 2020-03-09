#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

const vec2 positions[5] = vec2[5](
    vec2(-1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, -1.0)
);

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
