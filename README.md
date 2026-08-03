# Build and Development Commands
 
 - **Build project**: `cargo build`
 - **Build release**: `cargo build --release`
 - **Run project**: `cargo run`
 - **Check compilation**: `cargo check`
 - **Run with logging**: `RUST_LOG=debug cargo run`
 
 ---
 
 # Project Architecture
 
 This is a 3D game engine built in Rust using `wgpu` for GPU rendering. The project has evolved from a primitive showcase into a full game framework with entity management, combat systems, and advanced graphics features.
 
 ## Core Architecture
 
 The application follows a layered entity-component-like architecture with distinct separation between engine, graphics, input, and game systems:
 
 ### Main Entry Point (`src/main.rs`)
 - Initializes logging and creates the `WindowManager`.
 - A simple entry point that delegates directly to the engine layer.
 
 ### Engine Layer (`src/engine/`)
 - **WindowManager**: Manages the window lifecycle and main render loop using `winit`.
 - **Engine**: The core engine coordinator handling phased update and render cycles.
 - **Scheduler**: The game system scheduler that manages entity updates (player, enemies, bullets).
 - **Math**: Vector math utilities and common types (`Vec3`, `Transform`).
 - **Time**: Delta time tracking and FPS monitoring.
 
 ### Graphics Layer (`src/graphics/`)
 - **GraphicsEngine**: The main renderer managing the `wgpu` context, camera, and render pipelines.
 - **ThirdPersonCamera**: An orbital camera system supporting both manual and auto-rotation modes.
 - **Line Rendering**: A specialized instanced line renderer for wireframe primitives.
 - **Lightning Effects**: An advanced electrical discharge system with fractal branching and seeking tendrils.
 - **Advanced Skybox System**: Multi-fractal rendering with dynamic transformation networks and wave interference.
 - **Primitive System**: Supports various 3D primitive types (cubes, spheres, etc.).
 - **Advanced Features**: Bloom renderer, frustum culling, and primitive caching.
 - **Shaders**: Custom WGSL shaders for various rendering techniques.
 
 ### Input Layer (`src/input/`)
 - **InputManager**: The central input coordinator handling keyboard, mouse, and gamepad inputs.
 - **Action System**: Maps raw inputs to semantic actions (e.g., `MoveForward`, `Fire`).
 - **Multi-device Support**: A unified interface for keyboard/WASD, mouse, and gamepad controls.
 - **Gamepad Integration**: Uses the `gilrs` crate for cross-platform gamepad support.
 
 ### Scene/Game Layer (`src/scene/`)
 - **Player System**: Manages the player entity, movement, health, and weapon management.
 - **Enemy System**: Modular enemy entities with distinct AI behaviors:
   - **Heavy**: Slow, tank-like enemies.
   - **Chaser**: Fast pursuit enemies.
   - **Drone**: Basic patrol enemies.
   - **Splitter**: Enemies that split into smaller units upon death.
   - **Cannibal**: Enemies that consume other enemies for power.
   - **Shield**: Enemies protected by orbiting shield spheres.
   - **Snake**: Multi-segmented snake enemies with following behavior.
   - **Blob**: Network-connected enemies with a core and surrounding nodes.
 - **Bullet System**: Projectile physics supporting bullet-to-bullet collision and deflection.
 - **Weapon System**: 9 distinct weapon types with unique mechanics:
   - **BasicBlaster**: Standard rapid-fire energy weapon.
   - **Shotgun**: 100-pellet spread weapon with a 3D spherical cone distribution.
   - **AntiGravityCannon**: Negative-mass projectiles repelled by gravity.
   - **ChainLightning**: Arcing lightning that jumps dynamically between enemies.
   - **SeekingExplosive**: Homing missiles with an explosive splash radius.
   - **ImplosionLauncher**: Large-radius implosion bombs that pull entities inward.
   - **FractalCannon**: Bullets that spawn fractal generations of child projectiles.
   - **LaserCannon**: Ultra-fast beam weapon with trailing visual effects.
   - **LargeBodyLauncher**: Fires gravitational asteroid masses as projectiles.
 - **Large Body System**: Handles gravitational celestial bodies (planets, stars, black holes, etc.).
 - **Physics System**: GPU-accelerated gravitational forces and N-body simulation.
 
 ---
 
 # Key Design Patterns
 
 - **Entity Management**: Each game entity type has a dedicated manager handling spawning, updating, and lifecycle.
 - **Phased Update Cycle**: The engine uses distinct phases (`pre-update`, `update`, `post-update`, `pre-render`, `post-render`) for organized system execution.
 - **Input Action Mapping**: Raw input events translate to semantic actions through the `ActionBindings` system, providing consistent controls across devices.
 - **Camera-Relative Movement**: Player movement is calculated relative to the camera's orientation for intuitive 3D controls.
 - **Instanced Rendering**: The graphics system uses GPU instancing to efficiently render multiple primitive instances with minimal draw calls.
 - **GPU-Accelerated Physics**: Gravitational forces and collision detection are computed on the GPU for high-performance simulation of large entity counts.
 - **Event-Driven Collision System**: Collision detection generates events that trigger particle effects, physics responses, and gameplay mechanics.
 - **Deferred Processing**: Collision responses (like velocity changes and entity removals) are queued and applied in batches to avoid borrow checker conflicts.
 
 ---
 
 # Key Dependencies
 
 - **`wgpu 26.0.1`**: Modern GPU API for cross-platform graphics rendering.
 - **`winit 0.30.12`**: Cross-platform window creation and event handling.
 - **`nalgebra 0.34.0`**: Linear algebra and 3D math operations.
 - **`gilrs 0.11.0`**: Gamepad input support.
 - **`rayon 1.11.0`**: Data parallelism for multi-threaded operations.
 
 ---
 
 # Development Notes
 
 - The project uses the **Rust 2024 edition** and includes comprehensive gamepad support with automatic keyboard fallback when gamepads are unavailable.
 - The codebase is architected as a modular game engine with clear separation between rendering, input, and game logic systems.
 - The engine supports advanced rendering features, including bloom effects, frustum culling, and efficient instanced rendering for large numbers of entities.
 
 ---
 
 # Advanced Features
 
 ## Collision System
 - **GPU Collision Detection**: High-performance collision detection offloaded to GPU compute shaders.
 - **Multi-Type Interactions**: Handles bullet-enemy, bullet-large body, enemy-large body, and bullet-bullet collisions.
 - **Collision Masks**: Efficient filtering system to determine which entity types can collide.
 - **Deferred Response Processing**: Collision responses are queued and applied in batches for optimal performance.
 
 ## Physics Simulation
 - **Gravitational N-Body**: Real gravitational physics affecting all entities based on mass and distance.
 - **GPU-Accelerated Forces**: Force calculations performed in parallel on GPU compute shaders.
 - **Orbital Mechanics**: Bullets and enemies follow realistic trajectories around celestial bodies.
 - **Bullet-Bullet Deflection**: Elastic collision physics between projectiles for emergent gameplay.
 
 ## Large Body System
 A complete celestial mechanics simulation system with emergent lifecycle dynamics, located in `src/scene/large_body/` as a modular sub-system.
 
 ### Module Structure
 - **`body_type.rs`** (196 lines): Type definitions, defaults, and death states.
 - **`body.rs`** (670 lines): Individual body lifecycle, physics integration, and death sequences.
 - **`spawner.rs`** (256 lines): Automatic population management and asteroid showers.
 - **`manager.rs`** (632 lines): Collection management, spawn coordination, and absorption processing.
 - **`mod.rs`**: Public API re-exports.
 
 ### 10 Celestial Body Types
 *Exotic types have distinct mass, radius, color, collision characteristics, angular velocity, and ergosphere effects:*
 - `BlackHole`, `BlackHoleLarge`, `WhiteHole`, `Star`, `NeutronStar`, `ExoticMatter`, `GasGiant`, `Planet`, `LauncherMass`, `Debug`.
 - `WhiteHole` uses negative mass for repulsive gravitational effects.
 - `BlackHoleLarge` is special and can absorb other bodies on collision.
 
 ### Lifecycle State Machine
 - **Lifecycle phases**: `Alive` $\rightarrow$ `DeathSequence` $\rightarrow$ `ReadyForRemoval`.
 - **Lifetimes**: 20–60s for spawned bodies, custom duration for death-spawned remnants.
 - **Supernova / Evaporation**: Animated radius changes during death sequences using quadratic easing:
   - **Star**: 5.0x expansion (supernova).
   - **BlackHole**: 2.5x expansion (Hawking radiation).
   - **BlackHoleLarge**: 4.0x expansion (supermassive evaporation).
   - **NeutronStar**: 0.5x collapse (implosion).
   - **GasGiant**: 3.0x expansion.
   - **Planet**: 1.5x expansion.
   - **ExoticMatter**: 8.0x expansion (annihilation).
 - **Distance Culling**: Bodies are culled at 5000 units from the origin.
 
 ### Death-Spawning System
 - Bodies can spawn new remnants during death sequences (e.g., `Star` $\rightarrow$ `NeutronStar` $\rightarrow$ `BlackHole`).
 - Remnants inherit their parent's position and partial velocity.
 - Queued processing prevents physics index conflicts.
 
 ### BlackHoleLarge Absorption
 - Absorbs any body on collision (triggered via the collision dispatcher).
 - Absorbs 100% of the victim's mass (using absolute values to handle negative mass).
 - Radius increases by the victim's full radius.
 - Gaining mass reduces the black hole's remaining lifetime by -2s per absorption, creating a natural instability cycle (spawn $\rightarrow$ absorb $\rightarrow$ grow $\rightarrow$ destabilize $\rightarrow$ spectacular death).
 - Emits dramatic particle effects (300 particles pulled toward the center).
 
 ### Automatic Spawner
 - Maintains 3–10 active bodies (target: 6) with a 4s spawn interval.
 - Randomly spawns bodies from the enabled pool (excludes `LauncherMass`, which is shower-only).
 - Spawn radius: 100–1000 units (avoids spawning within 100 units of the player).
 - **Asteroid Showers**: 5% chance to trigger a shower event spawning 20–60 `LauncherMass` bodies from a random direction 1000 units away.
 
 ### Visual Effects
 - **Atmospheric Rendering**: Transparency layers for `Star`, `GasGiant`, `Planet`, `NeutronStar`, and `BlackHole`.
 - **Trails**: Particle trails following each body (1 particle every 20ms, 15s lifetime).
 - **Solar Winds**: Periodic emissions (Stars: 6s, WhiteHoles: 2s, NeutronStars: 8s, ExoticMatter: 0.2s).
 
 ### Physics Integration
 - Gravitational synchronization with the GPU physics system every frame.
 - Angular velocity updates dynamically from orbital physics.
 - Frame-dragging (ergosphere) calculations for spinning bodies (`BlackHole`, `NeutronStar`).
 
 ## Dynamic Lighting System
 An atmospheric lighting system using positive and negative light sources that affect the skybox, located in `src/graphics/lighting.rs`.
 
 - **Uniform Lighting**: Light intensity calculated from camera position and applied uniformly to the skybox.
 - **Positive Lights (Brightness Emitters)**:
   - **Stars**: Intense warm golden light (intensity: 0.8, radius: 15× body radius).
   - **WhiteHoles**: Bright repulsive white light (intensity: 0.6, radius: 12×).
   - **NeutronStars**: Harsh blue-white light (intensity: 0.5, radius: 10×).
   - **Planets / GasGiants**: Soft reflected colored light (intensity: 0.15, radius: 5×).
 - **Negative Lights (Darkness Emitters)**:
   - **BlackHoles**: Darkness zones with a subtle blue tint (intensity: -0.6, radius: 20×).
   - **BlackHoleLarge**: Intense supermassive darkness (intensity: -1.0, radius: 25×).
   - **ExoticMatter**: Flickering purple-tinted darkness (intensity: -0.4 pulsing, radius: 12×).
 - **Technical Implementation**:
   - Supports up to 32 concurrent light sources (`MAX_LIGHTS`).
   - GPU uniform buffer updated every frame. Inverse-square falloff with smoothstep edge fading.
   - Light intensity scales dynamically with body mass.
 - **Shader Integration (`shaders/skybox.wgsl`)**:
   - Group 2 binding for lighting uniforms (32 lights × 32 bytes).
   - Per-fragment lighting accumulation loop. Signed values allow negative lights to subtract from scene brightness.

 ## Particle Effects
 - **Collision Particles**: Unique particle profiles based on collision types:
   - **Enemy Destruction**: 100 cyan particles, 1.0s lifetime.
   - **Bullet vs. Large Body**: 10 yellow particles, 0.8s lifetime.
   - **Bullet vs. Bullet**: 5 white particles, 0.6s lifetime.
   - **Large Body vs. Large Body**: 200 orange particles, 2.0s lifetime.
 - **GPU Particle System**: High-efficiency rendering with configurable counts, lifetimes, and colors.

 ## Advanced Physics Effects
 - **Angular Momentum**: Dynamic spinning calculations.
 - **Frame-Dragging (Ergosphere)**: Tangential forces applied within the ergosphere radius of rotating black holes and neutron stars.
 - **Collision Physics**: Elastic collisions conserving angular momentum.
 - **Arena Containment**: Soft binding forces keeping all celestial bodies inside the playable area.

 ## Lightning Effects System
 - **Fractal Branching**: Multi-generational branching with decay rates.
 - **Seeking Tendrils**: Dead-end exploratory branches for authentic electrical discharge visuals.
 - **True 3D Effects**: Full Y-axis branching for three-dimensional bolts.
 - **Visuals**: Multi-layered core/glow rendering with progressive formation animation.
 - **Testing**: Integrates test hotkey (Press `K` to spawn lightning bolts instantly).

 ## Advanced Skybox System
 - **Multi-Fractal Rendering**: Real-time rendering of Mandelbrot, Julia, Burning Ship, and Newton fractals.
 - **Transformation Network**: 12 dynamic transition pathways between fractals using spatial influence zones and time-varying weights.
 - **Wave Interference**: Mathematical standing wave systems (amplification tuned to 2.5).
 - **Golden Ratio Spiral**: Emerges at high-energy interference nodes (intensity: 0.08).
 - **Aesthetics**: Muted pastel colors for a contemplative cosmic atmosphere.
 - **Neural Filament Networks**: Ethereal cosmic threads connecting fractal boundaries.
 - **Perspective Distortion**: Spherical 3D projection adjusting for proper depth and curvature.

 ---

 # Performance Characteristics

 - **Scalable Architecture**: Maintains **75+ FPS** with hundreds of active entities and active physics calculations.
 - **Batch Processing**: Collision detection, physics updates, and particle spawning are executed in batch loops.
 - **GPU Offloading**: Heavy computational routines (N-body gravity, skybox fractals) run inside parallel GPU shaders.
 - **Lightning Rendering**: Instanced line rendering supports complex branching with minimal draw calls.
