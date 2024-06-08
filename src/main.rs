use app::App;
use tokio::runtime::Builder;
use winit::event_loop::EventLoop;

mod app;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let event_loop = EventLoop::new().unwrap();
    let rt = Builder::new_current_thread().build().unwrap();
    let mut app = App {
        window: None,
        runtime: rt,
        render_pipeline: None,
        surface: None,
        surface_config: None,
        device: None,
        queue: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
