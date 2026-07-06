# Bi83-kit Plugin Generation Instructions for AI Agents

**Target Audience:** LLMs, AI Coding Assistants, and Autonomous Agents.
**Context:** The user is asking you to generate a WebAssembly plugin for the Bi83 Emulator. 
**SDK Name:** `bi83-kit` (version: `0.1.0`)

When a user asks you to write a plugin for their emulator, you MUST strictly adhere to the following rules to ensure the generated Rust code compiles and runs efficiently using the `Bi83-kit` zero-cost abstraction SDK.

## CRITICAL RULES

1. **NO Boilerplate Memory Management:**
   - Do NOT write `alloc`, `dealloc`, or pointer casting functions. The `Bi83-kit` handles all of this internally.
   - Do NOT use `#![no_std]` unless explicitly requested. The emulator supports standard Rust allocations for `wasm32-unknown-unknown`.

2. **Use the `Plugin` Trait:**
   - Every plugin must be a struct that implements the `bi83_kit::Plugin` trait.
   - The trait has two methods: `fn init()` (called once) and `fn update(state: &mut bi83_kit::EmulatorState)` (called every frame).
   - You MUST export the plugin struct using the `bi83_kit::export_plugin!(YourStructName);` macro at the bottom of the file.

3. **Input Injection (State Mutation):**
   - The `EmulatorState` passed to `update` is mutable (`&mut`). 
   - If the user wants to simulate a button press, modify `state.keys` directly.
   - Example: `state.keys = 0xFF;`

4. **Available FFI Hooks:**
   - Always import these from `bi83_kit` as needed:
     - `log(msg: &str)`: For debugging.
     - `read_ram(addr: u32) -> u8`: For reading emulator RAM.
     - `write_ram(addr: u32, value: u8)`: For modifying emulator RAM (cheats, mods).
     - `draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32)`: For rendering UI overlays (Color format: `0xRRGGBBAA`).

## PERFECT PLUGIN TEMPLATE

When generating a plugin, use this exact structure:

```toml
# Cargo.toml
[package]
name = "my_ai_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
bi83-kit = "0.1.0"
```

```rust
// src/lib.rs
use bi83_kit::{export_plugin, log, read_ram, write_ram, draw_rect, EmulatorState, Plugin};

struct AIPlugin;

impl Plugin for AIPlugin {
    fn init() {
        log("AI Plugin successfully loaded!");
    }

    fn update(state: &mut EmulatorState) {
        // AI: Insert user logic here.
        // Example: Lock health to 100 at RAM address 0x4000
        let health = read_ram(0x4000);
        if health < 100 {
            write_ram(0x4000, 100);
        }

        // Example: Draw overlay
        draw_rect(10, 10, 50, 50, 0xFF0000FF);
    }
}

export_plugin!(AIPlugin);
```

## Compilation Note
Always remind the user to compile their plugin using:
`cargo build --target wasm32-unknown-unknown --release`
