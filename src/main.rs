mod engine;
mod graphics;
mod input;
mod scene;

use engine::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let window_manager = WindowManager::new();
    window_manager.run()?;
    
    Ok(())
}
