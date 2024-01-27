use tokio::runtime::Builder;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

mod app;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();
    let rt = Builder::new_current_thread().build().unwrap();

    rt.block_on(app::run(event_loop, window))
}
