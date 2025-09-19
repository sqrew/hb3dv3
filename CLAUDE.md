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
- **Enemy System**: Enemy entities with AI and behavior
- **Bullet System**: Projectile physics with bullet-bullet collision and deflection
- **Weapon System**: Weapon management and firing mechanics
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
- **9 Celestial Body Types**: BlackHole, WhiteHole, Star, Planet, NeutronStar, GasGiant, Asteroid, SpaceStation, Moon
- **Individual Properties**: Each type has distinct mass, radius, color, and collision characteristics  
- **Exotic Matter Physics**: WhiteHoles have negative mass, creating repulsive gravitational effects
- **Ratio-Based Collision**: Visual radius vs collision radius separation for gameplay tuning (e.g., 0.75 ratio for BlackHoles)
- **Physics Integration**: Large bodies participate in N-body gravitational simulation with proper negative mass handling

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

### Performance Characteristics
- **Scalable Architecture**: Maintains 75+ FPS even with hundreds of entities and complex physics
- **Batch Processing**: Collision detection, physics updates, and particle spawning performed in batches
- **GPU Utilization**: Heavy computational work offloaded to GPU for maximum performance
- **Lightning Performance**: Efficient line instancing supports complex lightning with 60+ segments and multiple branches