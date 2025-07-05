fn main() -> anyhow::Result<()> {
    pollster::block_on(mandelbrot_wgpu::run())
}
