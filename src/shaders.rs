use wgpu::Device;
use wgpu::ShaderModule;

pub fn vertex_shader_module(device: &Device) -> ShaderModule {
    let glsl = include_str!("shader.vert");
    let vs =
        wgpu::read_spirv(glsl_to_spirv::compile(glsl, glsl_to_spirv::ShaderType::Vertex).unwrap())
            .unwrap();

    device.create_shader_module(&vs)
}

pub fn fragment_shader_module(device: &Device) -> ShaderModule {
    let glsl_str = include_str!("shader.frag");
    let glsl = if cfg!(feature = "double") {
        glsl_str.replace("float", "double").replace("vec2", "dvec2")
    } else {
        glsl_str.to_owned()
    };

    let fs = wgpu::read_spirv(
        glsl_to_spirv::compile(&glsl, glsl_to_spirv::ShaderType::Fragment).unwrap(),
    )
    .unwrap();

    device.create_shader_module(&fs)
}
