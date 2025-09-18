mod engine;
mod graphics;
mod input;
mod scene;
mod ui;

use engine::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let window_manager = WindowManager::new();
    window_manager.run()?;
    
    Ok(())
}
