use std::fmt::{self, Display};
use std::str::FromStr;
use winit::event;
use zerocopy::AsBytes;

macro_rules! next_arg {
    ($args:tt, $type:ty) => {
        $args.next().map(parse_or_usage::<$type, String>).unwrap()
    };
}

const ZOOM_FACTOR: f64 = 1.05;
const TRANSFORM_STEP: f64 = 0.01;

#[cfg(not(feature = "double"))]
type GPUFloat = f32;
#[cfg(feature = "double")]
type GPUFloat = f64;

#[non_exhaustive]
pub struct State {
    pub window_size: (u32, u32),
    pub max_iterations: u32,
    pub scale: f64,
    pub center: (f64, f64),
}

impl State {
    pub fn from_args() -> Self {
        let mut args = std::env::args();
        args.next();

        match args.len() {
            5 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f64), next_arg!(args, f64));
                let window_size = (next_arg!(args, u32), next_arg!(args, u32));

                State {
                    window_size,
                    max_iterations,
                    center,
                    ..Default::default()
                }
            }
            3 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f64), next_arg!(args, f64));

                State {
                    max_iterations,
                    center,
                    ..Default::default()
                }
            }
            1 => State {
                max_iterations: next_arg!(args, u32),
                ..Default::default()
            },
            0 => State::default(),
            _ => usage(),
        }
    }

    pub fn fragment_uniform(&self) -> FragmentUniform {
        FragmentUniform {
            screen_size: [
                self.window_size.0 as GPUFloat,
                self.window_size.1 as GPUFloat,
            ],
            center: [self.center.0 as GPUFloat, self.center.1 as GPUFloat],
            scale: self.scale as GPUFloat,
            max_iterations: self.max_iterations,
            _padding: 0,
        }
    }

    pub fn handle_input(&mut self, key_code: event::VirtualKeyCode) -> bool {
        let mut redraw_needed = true;
        let step = TRANSFORM_STEP * self.scale;

        match key_code {
            event::VirtualKeyCode::A => self.center.0 -= step,
            event::VirtualKeyCode::D => self.center.0 += step,
            event::VirtualKeyCode::W => self.center.1 += step,
            event::VirtualKeyCode::S => self.center.1 -= step,
            event::VirtualKeyCode::Up => self.scale /= ZOOM_FACTOR,
            event::VirtualKeyCode::Down => self.scale *= ZOOM_FACTOR,
            event::VirtualKeyCode::Left => {
                if self.max_iterations > 200 {
                    self.max_iterations -= 200
                }
            }
            event::VirtualKeyCode::Right => self.max_iterations += 200,
            event::VirtualKeyCode::I => println!("{}", self),
            _ => redraw_needed = false,
        }

        redraw_needed
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            window_size: (800, 600),
            max_iterations: 200,
            scale: 2.0,
            center: (-0.5, 0.0),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Center: {:?}, Scale: {}, Iterations: {}",
            self.center, self.scale, self.max_iterations
        )
    }
}

#[derive(Copy, Clone, AsBytes)]
#[repr(C)]
pub struct FragmentUniform {
    screen_size: [GPUFloat; 2],
    center: [GPUFloat; 2],
    scale: GPUFloat,
    max_iterations: u32,
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
    println!(
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
