use std::{error::Error, fs};

use shaderc::{Compiler, ShaderKind};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    let mut compiler = Compiler::new().expect("could not create compiler");

    for entry in WalkDir::new("src").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path
                .extension()
                .map(|e| e.to_str().expect("could not convert extension to string"))
            {
                let shader_kind = match ext {
                    "vert" => Some(ShaderKind::Vertex),
                    "frag" => Some(ShaderKind::Fragment),
                    _ => None,
                };

                if let Some(shader_kind) = shader_kind {
                    println!(
                        "cargo:rerun-if-changed={}",
                        path.as_os_str()
                            .to_str()
                            .expect("could not convert path to string")
                    );

                    let src = if cfg!(feature = "double") && shader_kind == ShaderKind::Fragment {
                        fs::read_to_string(path)?
                            .replace("precision highp float;", "")
                            .replace("float", "double")
                            .replace("vec2", "dvec2")
                    } else {
                        fs::read_to_string(path)?
                    };

                    let compiled = compiler.compile_into_spirv(
                        &src,
                        shader_kind,
                        path.to_str().expect("could not convert path to string"),
                        "main",
                        None,
                    )?;

                    fs::write(
                        path.with_extension(format!("{}.spv", ext)),
                        compiled.as_binary_u8(),
                    )?;
                }
            }
        }
    }

    Ok(())
}
