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

    // Ultimate API Expansion
    fn host_http_get(url_ptr: u32, url_len: u32, out_ptr: u32, max_len: u32) -> u32;
    fn host_ui_notify(title_ptr: u32, title_len: u32, body_ptr: u32, body_len: u32);
    fn host_storage_write(k_ptr: u32, k_len: u32, v_ptr: u32, v_len: u32);
    fn host_storage_read(k_ptr: u32, k_len: u32, out_ptr: u32, max_len: u32) -> u32;
    fn host_get_key_state(keycode: u32) -> u32;
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

/// Performs a synchronous HTTP GET request.
pub fn http_get(url: &str) -> String {
    let mut out_buf = vec![0u8; 1024 * 1024]; // 1MB buffer
    unsafe {
        let len = host_http_get(
            url.as_ptr() as u32,
            url.len() as u32,
            out_buf.as_mut_ptr() as u32,
            out_buf.len() as u32,
        );
        out_buf.set_len(len as usize);
        String::from_utf8_lossy(&out_buf).into_owned()
    }
}

/// Sends a UI Notification to the Host.
pub fn ui_notify(title: &str, body: &str) {
    unsafe {
        host_ui_notify(
            title.as_ptr() as u32,
            title.len() as u32,
            body.as_ptr() as u32,
            body.len() as u32,
        );
    }
}

/// Writes a string to the plugin's persistent isolated storage.
pub fn storage_write(key: &str, value: &str) {
    unsafe {
        host_storage_write(
            key.as_ptr() as u32,
            key.len() as u32,
            value.as_ptr() as u32,
            value.len() as u32,
        );
    }
}

/// Reads a string from the plugin's persistent isolated storage.
pub fn storage_read(key: &str) -> Option<String> {
    let mut out_buf = vec![0u8; 1024 * 64]; // 64KB max for a value
    unsafe {
        let len = host_storage_read(
            key.as_ptr() as u32,
            key.len() as u32,
            out_buf.as_mut_ptr() as u32,
            out_buf.len() as u32,
        );
        if len == 0 {
            return None;
        }
        out_buf.set_len(len as usize);
        Some(String::from_utf8_lossy(&out_buf).into_owned())
    }
}

/// Checks if a given key is currently pressed on the host machine.
pub fn is_key_pressed(keycode: u32) -> bool {
    unsafe { host_get_key_state(keycode) != 0 }
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
