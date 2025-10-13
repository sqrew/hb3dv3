# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

- **Build project**: `cargo build`
- **Build release**: `cargo build --release`
- **Run project**: `cargo run`
- **Check compilation**: `cargo check`
- **Run with logging**: `RUST_LOG=debug cargo run`

## Project Architecture

This is a 3D game engine built in Rust using wgpu for GPU rendering. The project has evolved from a primitive showcase into a full game framework with entity management, combat systems, and advanced graphics features.

### Core Architecture

The application follows a layered entity-component-like architecture with distinct separation between engine, graphics, input, and game systems:

**Main Entry Point** (`src/main.rs`):
- Initializes logging and creates the WindowManager
- Simple entry that delegates to the engine layer

**Engine Layer** (`src/engine/`):
- **WindowManager**: Manages window lifecycle and main render loop using winit
- **Engine**: Core engine coordinator with phased update/render cycles
- **Scheduler**: Game system scheduler that manages entity updates (player, enemies, bullets)
- **Math**: Vector math utilities and types (Vec3, Transform)
- **Time**: Delta time tracking and FPS monitoring

**Graphics Layer** (`src/graphics/`):
- **GraphicsEngine**: Main renderer managing wgpu context, camera, and render pipelines
- **ThirdPersonCamera**: Orbital camera system with manual and auto-rotation modes
- **Line Rendering**: Specialized instanced line renderer for wireframe primitives
- **Lightning Effects**: Advanced electrical discharge system with fractal branching and seeking tendrils
- **Advanced Skybox System**: Multi-fractal rendering with dynamic transformation networks and wave interference
- **Primitive System**: Supports various 3D primitive types (cubes, spheres, etc.)
- **Advanced Features**: Bloom renderer, frustum culling, and primitive caching
- **Shaders**: Custom WGSL shaders for various rendering techniques

**Input Layer** (`src/input/`):
- **InputManager**: Central input coordinator handling keyboard, mouse, and gamepad
- **Action System**: Maps raw input to semantic actions (e.g., MoveForward, Fire)
- **Multi-device Support**: Unified interface for keyboard/WASD, mouse, and gamepad controls
- **Gamepad Integration**: Uses gilrs crate for cross-platform gamepad support

**Scene/Game Layer** (`src/scene/`):
- **Player System**: Player entity with movement, health, and weapon management
- **Enemy System**: Modular enemy entities with distinct AI behaviors:
  - **Heavy**: Slow, tank-like enemies
  - **Chaser**: Fast pursuit enemies
  - **Drone**: Basic patrol enemies
  - **Splitter**: Enemies that split into smaller units on death
  - **Cannibal**: Enemies that consume other enemies for power
  - **Shield**: Enemies with shield orbs providing protection
  - **Snake**: Multi-segmented snake enemies with following behavior
  - **Blob**: Network enemies with core and connected nodes
- **Bullet System**: Projectile physics with bullet-bullet collision and deflection
- **Weapon System**: 9 distinct weapon types with unique mechanics:
  - **BasicBlaster**: Standard rapid-fire energy weapon
  - **Shotgun**: 100-pellet spread weapon with 3D spherical cone distribution
  - **AntiGravityCannon**: Negative-mass projectiles repelled by gravity
  - **ChainLightning**: Arcing lightning that jumps between enemies
  - **SeekingExplosive**: Homing missiles with explosive radius
  - **ImplosionLauncher**: Large-radius implosion bombs that pull entities inward
  - **FractalCannon**: Bullets that spawn fractal generations of child projectiles
  - **LaserCannon**: Ultra-fast beam weapon with trailing effects
  - **LargeBodyLauncher**: Fires gravitational asteroid masses as projectiles
- **Large Body System**: Gravitational celestial bodies (planets, stars, black holes, etc.)
- **Physics System**: GPU-accelerated gravitational forces and N-body simulation

### Key Design Patterns

**Entity Management**: Each game entity type has a dedicated manager that handles spawning, updating, and lifecycle.

**Phased Update Cycle**: The engine uses distinct phases (pre-update, update, post-update, pre-render, post-render) for organized system updates.

**Input Action Mapping**: Raw input events are translated to semantic actions through the `ActionBindings` system, providing consistent controls across different input devices.

**Camera-Relative Movement**: Player movement is calculated relative to the camera's orientation, providing intuitive 3D controls.

**Instanced Rendering**: The graphics system uses GPU instancing to efficiently render multiple instances of primitives with minimal draw calls.

**GPU-Accelerated Physics**: Gravitational forces and collision detection are computed on the GPU for high-performance simulation with large entity counts.

**Event-Driven Collision System**: Collision detection generates events that trigger particle effects, physics responses, and gameplay mechanics.

**Deferred Processing**: Collision responses like velocity changes and entity removals are queued and applied in batches to avoid borrowing conflicts.

### Key Dependencies

- **wgpu 26.0.1**: Modern GPU API for cross-platform graphics rendering
- **winit 0.30.12**: Cross-platform window creation and event handling
- **nalgebra 0.34.0**: Linear algebra and 3D math operations
- **gilrs 0.11.0**: Gamepad input support
- **rayon 1.11.0**: Data parallelism for multi-threaded operations

## Development Notes

The project uses Rust 2024 edition and includes comprehensive gamepad support with automatic fallback when gamepads are unavailable. The codebase is architected as a modular game engine with clear separation between rendering, input, and game logic systems.

The engine supports advanced rendering features including bloom effects, frustum culling, and efficient instanced rendering for large numbers of entities.

## Advanced Features

### Collision System
- **GPU Collision Detection**: High-performance collision detection using GPU compute shaders
- **Multi-Type Interactions**: Handles bullet-enemy, bullet-large body, enemy-large body, and bullet-bullet collisions
- **Collision Masks**: Efficient filtering system to determine which entity types can collide
- **Deferred Response Processing**: Collision responses are queued and applied in batches for optimal performance

### Physics Simulation  
- **Gravitational N-Body**: Real gravitational physics affecting all entities based on mass and distance
- **GPU-Accelerated Forces**: Force calculations performed in parallel on GPU compute shaders
- **Orbital Mechanics**: Bullets and enemies follow realistic trajectories around celestial bodies
- **Bullet-Bullet Deflection**: Elastic collision physics between projectiles for emergent gameplay

### Large Body System
A complete celestial mechanics simulation system with emergent lifecycle dynamics. Located in `src/scene/large_body/` as a modular sub-system.

**Module Structure:**
- `body_type.rs` (196 lines) - Type definitions, defaults, death states
- `body.rs` (670 lines) - Individual body lifecycle, physics integration, death sequences
- `spawner.rs` (256 lines) - Automatic population management and asteroid showers
- `manager.rs` (632 lines) - Collection management, spawn coordination, absorption processing
- `mod.rs` - Public API re-exports

**10 Celestial Body Types:**
- BlackHole, BlackHoleLarge, WhiteHole, Star, NeutronStar, ExoticMatter, GasGiant, Planet, LauncherMass, Debug
- Each type has distinct mass, radius, color, collision characteristics, angular velocity, and ergosphere effects
- WhiteHoles use negative mass for repulsive gravitational effects
- BlackHoleLarge is special - can absorb other bodies on collision

**Lifecycle System:**
- Age tracking with configurable lifetimes (20-60s from spawner, custom for death-spawned)
- Death state machine: Alive → DeathSequence → ReadyForRemoval
- **Animated death sequences**: Radius changes smoothly over death duration with quadratic easing
  - Star: 5.0x expansion (supernova)
  - BlackHole: 2.5x expansion (Hawking radiation)
  - BlackHoleLarge: 4.0x expansion (supermassive evaporation)
  - NeutronStar: 0.5x collapse (implosion)
  - GasGiant: 3.0x expansion, Planet: 1.5x expansion
  - ExoticMatter: 8.0x expansion (annihilation)
- Type-specific death sequence effects with explosions, particles, and shockwaves
- Distance culling at 5000 units from origin

**Death-Spawning System:**
- Bodies can spawn new bodies during death sequences (e.g., Star → NeutronStar)
- Spawned bodies inherit position and partial velocity from parent
- Configurable lifetimes for spawned remnants
- Queued processing prevents physics index conflicts
- Extensible for complex death chains (Star → NeutronStar → BlackHole)

**BlackHoleLarge Absorption:**
- Absorbs any body on collision (collision-based trigger via dispatcher)
- 100% of victim's mass gained (uses absolute value to handle negative mass)
- Radius increases by victim's full radius
- **-2s lifetime reduction per absorption** - grows more unstable with each meal
- Creates natural death cycle: spawn → absorb → grow → destabilize → spectacular death
- Tracks absorption count for gameplay feedback
- Dramatic particle effects at absorption point (300 particles pulled toward black hole)

**Automatic Spawner:**
- Maintains 3-10 bodies (target: 6) with 4s spawn interval
- Random body types from enabled pool (excludes LauncherMass - shower-only)
- Lifetimes: 20-60s randomized
- Spawn radius: 100-1000 units, avoids player within 100 units
- 5% chance for asteroid shower events instead of single spawn
- Configurable at runtime via spawner_mut()

**Asteroid Shower Events:**
- 20-60 LauncherMass bodies spawn from random direction 1000 units away
- Random speeds: 50-250 units/sec with 0.5 radian cone spread
- Each asteroid gets random lifetime multiplier (0.5-1.5x base)
- Creates dramatic temporary influx of gravitational masses
- Spherical coordinate generation for realistic trajectories

**Visual Effects:**
- Body-specific colors and primitive types (Icosahedron for LauncherMass, Spheres for others)
- Atmospheric rendering for Star, GasGiant, Planet, NeutronStar, BlackHole with transparency
- Particle trails following each body (1 particle every 20ms, 15s lifetime)
- Solar wind emissions (Stars: every 6s, WhiteHoles: 2s, NeutronStars: 8s, ExoticMatter: 0.2s)
- Death sequence explosions with type-specific colors and particle counts

**Physics Integration:**
- Each body maintains physics_index for N-body gravitational simulation
- Position/velocity synchronized with GPU physics system each frame
- Angular velocity updates from physics (spin changes from interactions)
- Careful index tracking when bodies are removed (all higher indices decremented)
- Frame-dragging effects for spinning bodies (BlackHole, NeutronStar)
- Ergosphere radius and strength calculated per body type

### Particle Effects
- **Collision-Triggered Particles**: Different particle effects for each collision type
  - Enemy destruction: 100 cyan particles, 1.0s lifetime
  - Bullet vs Large Body: 10 yellow particles, 0.8s lifetime  
  - Bullet vs Bullet: 5 white particles, 0.6s lifetime
  - Large Body vs Large Body: 200 orange particles, 2.0s lifetime
- **GPU Particle System**: Efficient particle rendering with configurable count, lifetime, and colors
- **Impact Point Calculation**: Particles spawn at precise collision locations

### Advanced Physics Effects
- **Angular Momentum**: Large bodies have rotation and angular velocity with visual spinning effects
- **Frame-Dragging (Ergosphere)**: Spinning black holes and neutron stars create tangential forces within ergosphere radius
- **Collision Physics**: Elastic collisions between large bodies with angular momentum conservation
- **Arena Containment**: Soft binding forces keep all bodies within playable area using absolute mass to handle negative mass correctly

### Lightning Effects System
- **Fractal Branching**: Multi-generational branching with configurable probability and decay rates
- **Seeking Tendrils**: Dead-end exploratory branches that create realistic electrical discharge patterns
- **True 3D Effects**: Full Y-axis branching for authentic three-dimensional lightning
- **Advanced Visual Features**: Multi-layered rendering with core/glow effects, progressive formation animation
- **Highly Configurable**: Extensive configuration options for segment count, chaos, thickness, duration, colors
- **Performance Optimized**: GPU-efficient line rendering with configurable complexity levels
- **Test Integration**: Press 'K' to spawn lightning bolts for testing and demonstration

### Advanced Skybox System
- **Multi-Fractal Rendering**: Real-time rendering of Mandelbrot, Julia, Burning Ship, and Newton fractals
- **Multi-Juncture Transformation Network**: Non-linear fractal metamorphosis preventing predictable cyclic patterns
  - 12 dynamic transformation pathways (each fractal can transform to any of the other 3)
  - Probability-based transitions with spatial influence zones and time-varying weights
  - Sharp transition thresholds using smoothstep functions for crisp mathematical boundaries
- **Wave Interference Patterns**: Mathematical standing wave systems creating geometric enhancement
  - Golden ratio spiral emergence at high-energy interference nodes
  - Spatial frequency modulation with distance-based phase calculations
  - 30% slower pulsing timing for contemplative viewing experience
- **Muted Pastel Aesthetics**: Carefully tuned brightness and intensity for serene visual experience
  - Softened wave interference (reduced from 4.0 to 2.5 amplification)
  - Reduced golden spiral intensity (0.15 to 0.08) for subtle geometric patterns
  - Gentle color enhancement preserving mathematical beauty while maintaining calm atmosphere
- **Neural Filament Networks**: Ethereal cosmic threads connecting fractal boundary regions
- **Spherical Perspective Distortion**: 3D fractal projection with proper depth and curvature
- **Sharp Boundary Enhancement**: Crisp edge definition for precise mathematical visualization
- **Performance Optimized**: Complex real-time fractal mathematics running at 60+ FPS

### Performance Characteristics
- **Scalable Architecture**: Maintains 75+ FPS even with hundreds of entities and complex physics
- **Batch Processing**: Collision detection, physics updates, and particle spawning performed in batches
- **GPU Utilization**: Heavy computational work offloaded to GPU for maximum performance
- **Lightning Performance**: Efficient line instancing supports complex lightning with 60+ segments and multiple branches
- **Fractal Performance**: Real-time multi-fractal skybox with interference patterns at optimal frame rates