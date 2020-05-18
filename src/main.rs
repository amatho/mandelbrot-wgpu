fn main() {
    futures::executor::block_on(mandelbrot_wgpu::run());
}
