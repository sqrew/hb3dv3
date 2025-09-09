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
- **Bullet System**: Projectile physics and collision handling
- **Weapon System**: Weapon management and firing mechanics

### Key Design Patterns

**Entity Management**: Each game entity type has a dedicated manager that handles spawning, updating, and lifecycle.

**Phased Update Cycle**: The engine uses distinct phases (pre-update, update, post-update, pre-render, post-render) for organized system updates.

**Input Action Mapping**: Raw input events are translated to semantic actions through the `ActionBindings` system, providing consistent controls across different input devices.

**Camera-Relative Movement**: Player movement is calculated relative to the camera's orientation, providing intuitive 3D controls.

**Instanced Rendering**: The graphics system uses GPU instancing to efficiently render multiple instances of primitives with minimal draw calls.

### Key Dependencies

- **wgpu 26.0.1**: Modern GPU API for cross-platform graphics rendering
- **winit 0.30.12**: Cross-platform window creation and event handling
- **nalgebra 0.34.0**: Linear algebra and 3D math operations
- **gilrs 0.11.0**: Gamepad input support
- **rayon 1.11.0**: Data parallelism for multi-threaded operations

## Development Notes

The project uses Rust 2024 edition and includes comprehensive gamepad support with automatic fallback when gamepads are unavailable. The codebase is architected as a modular game engine with clear separation between rendering, input, and game logic systems.

The engine supports advanced rendering features including bloom effects, frustum culling, and efficient instanced rendering for large numbers of entities.