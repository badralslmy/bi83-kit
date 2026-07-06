# Bi83-kit API Documentation

Welcome to the comprehensive API documentation for the `Bi83-kit` SDK. This guide covers all the functions and structs exposed by the SDK to help you build powerful WebAssembly plugins for the Bismuth Emulator.

## Core Concepts

The SDK is built around a single trait: `Plugin`. By implementing this trait on your struct, and calling `export_plugin!(YourStruct)`, the SDK automatically generates the necessary `extern "C"` FFI functions (`plugin_init` and `plugin_update`) and handles memory allocations via `bytemuck`.

### `EmulatorState`

The `EmulatorState` struct is passed to your `update` function every frame. It represents the state of the emulator.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EmulatorState {
    /// Time in seconds since the last frame (e.g., 0.016 for 60FPS).
    pub delta_time: f32, 
    
    /// The total number of frames processed since the emulator started.
    pub frame_count: u32, 
    
    /// A bitmask of currently pressed keys/buttons.
    /// Because the state is passed as `&mut EmulatorState`, modifying this value 
    /// will inject inputs back into the emulator (useful for TAS, Auto-Fire, etc.).
    pub keys: u32,       
}
```

## Functions

### `log(msg: &str)`
Sends a string message to the host emulator's standard output (console).
* **Usage**: Ideal for debugging your plugin since standard `println!` does not work in Wasm without WASI.
* **Example**: `log("Player entered the boss room!");`

### `read_ram(address: u32) -> u8`
Reads a single byte from the emulator's memory space.
* **Arguments**: `address` - The 32-bit physical address in the emulated RAM.
* **Returns**: The byte (`u8`) at the specified address.
* **Usage**: Useful for memory scanners, reading player health, or fetching current game variables.

### `write_ram(address: u32, value: u8)`
Writes a single byte directly to the emulator's memory space.
* **Arguments**: 
  - `address` - The physical address to write to.
  - `value` - The byte to write.
* **Usage**: Used for cheating (locking health to 100), ROM hacking on-the-fly, or modifying game logic.

### `draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32)`
Sends a drawing command to the host emulator to render a rectangle over the game's display.
* **Arguments**:
  - `x`, `y`: The top-left coordinates of the rectangle.
  - `w`, `h`: The width and height of the rectangle.
  - `color`: The 32-bit color in `0xRRGGBBAA` format (e.g., Red is `0xFF0000FF`).
* **Usage**: Useful for drawing hitboxes, custom plugin UIs, or highlighting objects on the screen.

### `http_get(url: &str) -> String`
*(Legacy)* Performs a synchronous HTTP GET request and returns the response body as a string. Note: Synchronous calls may freeze the emulator. Prefer `http_request_async`.

### `http_request_async(url: &str, method: &str, headers: &str, body: Option<&[u8]>) -> u32`
Initiates a non-blocking asynchronous HTTP request. 
* **Arguments**:
  - `url` - The destination URL.
  - `method` - HTTP method (e.g., `GET`, `POST`).
  - `headers` - Newline-separated headers (e.g., `Client-ID: abc\nAuthorization: Bearer xyz`).
  - `body` - Optional byte slice for request body.
* **Returns**: A request ID (`u32`) to be used with `http_poll_response`.

### `http_poll_response(req_id: u32) -> Option<Result<Vec<u8>, String>>`
Polls the status of an asynchronous HTTP request.
* **Returns**: 
  - `None` if the request is still pending.
  - `Some(Ok(Vec<u8>))` if the request succeeded.
  - `Some(Err(String))` if the request failed.

### `ui_inject_html(id: &str, html: &str)`
Injects raw HTML into the Bismuth Emulator's Tauri-based web frontend.
* **Usage**: Ideal for rendering rich UIs, banners, or images. The HTML will be rendered in an absolute overlay.

### `ui_remove_html(id: &str)`
Removes previously injected HTML from the UI using its unique `id`.

### `ui_notify(title: &str, body: &str)`
Sends a push notification to the Bismuth Emulator UI.
* **Usage**: Alerting the user when a macro is activated, or when metadata finishes downloading.
* **Example**: `ui_notify("Scraper", "Successfully downloaded 5 covers");`

### `storage_write(key: &str, value: &str)`
Writes a string into the plugin's dedicated, persistent sandbox storage.
* **Usage**: Saving user preferences, caching API tokens, or remembering last scrape timestamps.

### `storage_read(key: &str) -> Option<String>`
Reads a string from the plugin's storage.
* **Returns**: `Some(String)` if the key exists, `None` otherwise.

### `storage_write_bin(key: &str, value: &[u8])`
Writes binary data (such as image bytes) directly to persistent storage.

### `storage_read_bin(key: &str) -> Option<Vec<u8>>`
Reads binary data from persistent storage.
* **Returns**: `Some(Vec<u8>)` if the key exists, `None` otherwise.

### `is_key_pressed(keycode: u32) -> bool`
Checks if a specific physical key on the keyboard/controller is currently pressed.
* **Usage**: Triggering cheat codes or speedrun timers via hotkeys.

## Best Practices

1. **Performance**: Since `update` is called every frame (e.g., 60 times a second), avoid heavy computations, loops, or allocations inside it unless absolutely necessary.
2. **State Management**: If your plugin needs to track variables across frames, store them as `static mut` (with `unsafe`) or use `std::cell::RefCell` wrapped in a `thread_local!`. Since Wasm is single-threaded, this is perfectly safe.
