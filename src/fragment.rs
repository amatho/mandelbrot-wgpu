use event::WindowEvent;
use std::fmt::{self, Display};
use std::str::FromStr;
use winit::{
    dpi::PhysicalSize,
    event::{self, ElementState, KeyboardInput},
};
use zerocopy::AsBytes;

macro_rules! next_arg {
    ($args:tt, $type:ty) => {
        $args.next().map(parse_or_usage::<$type, String>).unwrap()
    };
}

const ZOOM_FACTOR: f32 = 1.05;
const TRANSFORM_STEP: f32 = 0.01;

#[non_exhaustive]
pub struct FragmentState {
    pub size: PhysicalSize<u32>,
    pub max_iterations: u32,
    pub scale: f32,
    pub center: (f32, f32),
}

impl FragmentState {
    pub fn from_args() -> Self {
        let mut args = std::env::args();
        args.next();

        match args.len() {
            5 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f32), next_arg!(args, f32));
                let size = PhysicalSize::new(next_arg!(args, u32), next_arg!(args, u32));

                FragmentState {
                    size,
                    max_iterations,
                    center,
                    ..Default::default()
                }
            }
            3 => {
                let max_iterations = next_arg!(args, u32);
                let center = (next_arg!(args, f32), next_arg!(args, f32));

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

    pub fn fragment_uniform(&self) -> FragmentUniform {
        FragmentUniform {
            screen_size: [self.size.width as f32, self.size.height as f32],
            center: [self.center.0, self.center.1],
            scale: self.scale,
            max_iterations: self.max_iterations,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let step = TRANSFORM_STEP * self.scale;

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key_code {
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
                _ => return false,
            },
            _ => return false,
        }

        true
    }
}

impl Default for FragmentState {
    fn default() -> Self {
        FragmentState {
            size: PhysicalSize::new(800, 600),
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

#[derive(Copy, Clone, AsBytes)]
#[repr(C)]
pub struct FragmentUniform {
    screen_size: [f32; 2],
    center: [f32; 2],
    scale: f32,
    max_iterations: u32,
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
