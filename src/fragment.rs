use bytemuck::{Pod, Zeroable};
use std::fmt::{self, Display};
use std::str::FromStr;
use winit::keyboard::KeyCode;

macro_rules! next_arg {
    ($args:tt, $type:ty) => {
        $args.next().map(parse_or_usage::<$type, String>).unwrap()
    };
}

const ZOOM_FACTOR: f64 = 1.05;
const TRANSFORM_STEP: f64 = 0.01;

#[cfg(not(feature = "double"))]
type GpuFloat = f32;
#[cfg(feature = "double")]
type GpuFloat = f64;

#[non_exhaustive]
#[derive(Debug)]
pub struct FragmentState {
    pub size: (u32, u32),
    pub max_iterations: u32,
    pub scale: f64,
    pub center: (f64, f64),
}

impl FragmentState {
    pub fn from_args() -> Self {
        let mut args = std::env::args();
        args.next();

        match args.len() {
            5 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f64), next_arg!(args, f64));
                let size = (next_arg!(args, u32), next_arg!(args, u32));

                FragmentState {
                    size,
                    max_iterations,
                    center,
                    ..Default::default()
                }
            }
            3 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f64), next_arg!(args, f64));

                FragmentState {
                    max_iterations,
                    center,
                    ..Default::default()
                }
            }
            1 => FragmentState {
                max_iterations: next_arg!(args, u32),
                ..Default::default()
            },
            0 => FragmentState::default(),
            _ => usage(),
        }
    }

    pub fn handle_key_pressed(&mut self, code: KeyCode) {
        let step = TRANSFORM_STEP * self.scale;

        match code {
            KeyCode::KeyA => self.center.0 -= step,
            KeyCode::KeyD => self.center.0 += step,
            KeyCode::KeyW => self.center.1 += step,
            KeyCode::KeyS => self.center.1 -= step,
            KeyCode::ArrowUp => self.scale /= ZOOM_FACTOR,
            KeyCode::ArrowDown => self.scale *= ZOOM_FACTOR,
            KeyCode::ArrowLeft => self.max_iterations = self.max_iterations.saturating_sub(200),
            KeyCode::ArrowRight => self.max_iterations += 200,
            KeyCode::KeyI => log::info!("Fragment state: {}", self),
            _ => {}
        }
    }

    pub fn fragment_uniform(&self) -> FragmentUniform {
        let (width, height) = self.size;
        let (center_x, center_y) = self.center;
        FragmentUniform {
            screen_size: [width as GpuFloat, height as GpuFloat],
            center: [center_x as GpuFloat, center_y as GpuFloat],
            scale: self.scale as GpuFloat,
            max_iterations: self.max_iterations,
            _padding: 0,
        }
    }
}

impl Default for FragmentState {
    fn default() -> Self {
        FragmentState {
            size: (800, 600),
            max_iterations: 200,
            scale: 2.0,
            center: (-0.5, 0.0),
        }
    }
}

impl Display for FragmentState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Center: {:?}, Scale: {}, Iterations: {}",
            self.center, self.scale, self.max_iterations
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct FragmentUniform {
    screen_size: [GpuFloat; 2],
    center: [GpuFloat; 2],
    scale: GpuFloat,
    max_iterations: u32,
    // Required if using f64
    _padding: u32,
}

fn parse_or_usage<T, S>(s: S) -> T
where
    T: FromStr,
    S: AsRef<str>,
{
    s.as_ref().parse().unwrap_or_else(|_| usage())
}

fn usage() -> ! {
    eprintln!(
        "Usage:
    mandelbrot [<iterations> <center real> <center imag> <width> <height>]
    or
    mandelbrot [<iterations> <center real> <center imag>]
    or
    mandelbrot [<iterations>]
    "
    );
    std::process::exit(1);
}
