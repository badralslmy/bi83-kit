pub use bytemuck;
use bytemuck::{Pod, Zeroable};

// This matches the ABI defined in the host
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct EmulatorState {
    pub delta_time: f32,
    pub frame_count: u32,
    pub keys: u32,
}

unsafe extern "C" {
    // Host function import
    fn host_log(ptr: u32, len: u32);
    fn host_read_ram(addr: u32) -> u32;
    fn host_write_ram(addr: u32, val: u32);
    fn host_draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32);
}

/// Logs a message to the host emulator console.
pub fn log(msg: &str) {
    unsafe {
        host_log(msg.as_ptr() as u32, msg.len() as u32);
    }
}

/// Reads a byte from the Emulator's RAM.
pub fn read_ram(address: u32) -> u8 {
    unsafe {
        host_read_ram(address) as u8
    }
}

/// Writes a byte to the Emulator's RAM.
pub fn write_ram(address: u32, value: u8) {
    unsafe {
        host_write_ram(address, value as u32);
    }
}

/// Draws a colored rectangle overlay on the screen (Color is 0xRRGGBBAA).
pub fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    unsafe {
        host_draw_rect(x, y, w, h, color);
    }
}

// Memory allocation for the host to pass data
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: u32) -> u32 {
    let mut buf = Vec::<u8>::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    // Forget the vector so Rust doesn't free the memory
    std::mem::forget(buf);
    ptr as usize as u32
}

// Memory deallocation for the host to free data
#[unsafe(no_mangle)]
pub extern "C" fn dealloc(ptr: u32, len: u32) {
    unsafe {
        // Reconstruct the vector so Rust can free it
        let _ = Vec::from_raw_parts(ptr as *mut u8, 0, len as usize);
    }
}

/// The core Plugin trait that all Bi83 plugins must implement.
pub trait Plugin {
    /// Called once when the plugin is loaded by the emulator.
    fn init() {}

    /// Called on every frame/tick of the emulator.
    /// `state` is mutable, allowing you to inject inputs (e.g., `state.keys = 0xFF;`)
    fn update(_state: &mut EmulatorState) {}
}

/// A macro to export a plugin struct so the emulator can load it.
#[macro_export]
macro_rules! export_plugin {
    ($plugin:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_init() {
            <$plugin as $crate::Plugin>::init();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn plugin_update(state_ptr: u32) {
            let state: &mut $crate::EmulatorState = unsafe {
                let ptr = state_ptr as *mut $crate::EmulatorState;
                &mut *ptr
            };
            <$plugin as $crate::Plugin>::update(state);
        }
    };
}
