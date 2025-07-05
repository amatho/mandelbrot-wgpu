struct FragmentUniform {
    screen_size: vec2<f64>,
    center: vec2<f64>,
    scale: f64,
    max_iter: u32,
}

@group(0) @binding(0)
var<uniform> state: FragmentUniform;

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f64>) -> @location(0) vec4<f64> {
    let aspect_ratio = state.screen_size.x / state.screen_size.y;
    var coord: vec2<f64>;
    coord.x = ((frag_pos.x / state.screen_size.y) - 0.5 * aspect_ratio) * state.scale;
    coord.y = ((frag_pos.y / state.screen_size.y) - 0.5) * state.scale;

    let c = vec2<f64>(coord.x + state.center.x, coord.y - state.center.y);

    var z = c;
    var i: u32;
    for (i = 0; i < state.max_iter; i++) {
        let x = (z.x * z.x - z.y * z.y) + c.x;
        let y = (z.y * z.x + z.x * z.y) + c.y;

        if ((x * x + y * y) > 4.0) {
            break;
        }

        z.x = x;
        z.y = y;
    }

    if (i == state.max_iter) {
        return vec4<f64>(0.0, 0.0, 0.0, 1.0);
    } else {
        let hue = f64(i) / f64(state.max_iter);
        let saturation = 1.0;
        let value = 0.7;
        return vec4<f64>(hsv2rgb(vec3<f64>((hue + 0.75) % 1.0, saturation, value)), 1.0);
    }
}

fn hsv2rgb(c: vec3<f64>) -> vec3<f64> {
    let K = vec4<f64>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}
