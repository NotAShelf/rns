//! # RNS
//!
//! Provides FFI bindings to Neovim's Lua API, allowing for
//! the creation of Neovim plugins in Rust. This library handles the interaction
//! between Rust, Lua, and Neovim's C API.

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};

mod interop;
mod pman;
use interop::register_nvim_interop_functions;
use pman::register_plugin_functions;

// Platform-specific definitions
#[cfg(target_os = "macos")]
pub const LIB_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
pub const LIB_EXTENSION: &str = "so";
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub const LIB_EXTENSION: &str = "so"; // Default to .so for other platforms

/// Opaque representation of Lua state
#[repr(C)]
pub struct LuaState {
    _private: [u8; 0],
}

/// Error types that may occur during FFI operations
#[derive(Debug)]
pub enum Error {
    /// Encountered a null pointer where a valid pointer was expected
    NullPointer,
    /// Failed to convert between Rust and C strings
    StringConversion,
    /// Failed to execute a Neovim command
    CommandExecution,
}

type Result<T> = std::result::Result<T, Error>;

// FFI bindings to the Lua C API
extern "C" {
    fn lua_createtable(l: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_pushcclosure(l: *mut LuaState, f: extern "C" fn(*mut LuaState) -> c_int, n: c_int);
    fn lua_setfield(l: *mut LuaState, idx: c_int, k: *const c_char);
    fn luaL_checklstring(l: *mut LuaState, arg: c_int, len: *mut usize) -> *const c_char;
}

// FFI bindings to the external Neovim API
#[cfg(not(target_os = "macos"))]
extern "C" {
    pub fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    pub fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
    pub fn xfree(ptr: *mut CVoid);
}

// FFI bindings to the external Neovim API for macOS
#[cfg(target_os = "macos")]
extern "C" {
    #[link_name = "do_cmdline_cmd"]
    pub fn do_cmdline_cmd(cmd: *const c_char) -> c_int;

    #[link_name = "concat_str"]
    pub fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;

    #[link_name = "xfree"]
    pub fn xfree(ptr: *mut CVoid);
}

// Add a void pointer type for xfree
type CVoid = std::ffi::c_void;

/// Wrapper for Neovim-owned strings that ensures proper memory management
struct NeovimString {
    ptr: *mut c_char,
}

impl NeovimString {
    /// Creates a new `NeovimString` from a raw C string pointer
    ///
    /// # Safety
    ///
    /// The pointer must be a valid C string allocated by Neovim's memory
    /// management functions. This function does not check if the pointer is valid,
    /// and using an invalid pointer may lead to undefined behavior.
    const unsafe fn new(ptr: *mut c_char) -> Self {
        Self { ptr }
    }

    /// Converts the C string to a Rust String
    fn to_string(&self) -> Result<String> {
        if self.ptr.is_null() {
            return Err(Error::NullPointer);
        }

        unsafe {
            let cstr = CStr::from_ptr(self.ptr);
            Ok(cstr.to_string_lossy().into_owned())
        }
    }
}

impl Drop for NeovimString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                #[cfg(target_os = "macos")]
                {
                    // On macOS, ensure we're using the correct xfree
                    xfree(self.ptr.cast::<CVoid>());
                }

                #[cfg(not(target_os = "macos"))]
                xfree(self.ptr.cast::<CVoid>());
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

/// Retrieves a string from the Lua stack at the given index
fn lua_check_string(l: *mut LuaState, idx: c_int) -> Result<String> {
    unsafe {
        let mut len: usize = 0;
        let ptr = luaL_checklstring(l, idx, &mut len);
        if ptr.is_null() {
            Err(Error::NullPointer)
        } else {
            Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

/// Extracts a Rust String from a C string pointer
pub(crate) fn extract_c_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        return Err(Error::NullPointer);
    }

    unsafe { Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned()) }
}

/// Runs a Neovim command
pub(crate) fn run_cmd(cmd: &str) -> Result<()> {
    let c_cmd = CString::new(cmd).map_err(|_| Error::StringConversion)?;

    let result = unsafe { do_cmdline_cmd(c_cmd.as_ptr()) };

    if result == 0 {
        Ok(())
    } else {
        Err(Error::CommandExecution)
    }
}

/// Safe wrapper around Lua state pointer
pub struct Lua<'a> {
    state: *mut LuaState,
    _marker: PhantomData<&'a LuaState>,
}

impl Lua<'_> {
    /// Creates a new Lua wrapper from a raw `LuaState` pointer
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `LuaState` pointer is valid and remains
    /// valid for the lifetime of the returned Lua instance. The pointer must
    /// point to a properly initialized Lua state.
    pub const unsafe fn new(state: *mut LuaState) -> Result<Self> {
        if state.is_null() {
            return Err(Error::NullPointer);
        }

        Ok(Lua {
            state,
            _marker: PhantomData,
        })
    }

    /// Creates a new table on the Lua stack
    pub fn create_table(&self, narr: c_int, nrec: c_int) {
        unsafe {
            lua_createtable(self.state, narr, nrec);
        }
    }

    /// Pushes a C closure onto the Lua stack
    pub fn push_cclosure(&self, f: extern "C" fn(*mut LuaState) -> c_int, n: c_int) {
        unsafe {
            lua_pushcclosure(self.state, f, n);
        }
    }

    /// Sets a field in the table at the given index
    pub fn set_field(&self, idx: c_int, k: &str) -> Result<()> {
        let c_key = CString::new(k).map_err(|_| Error::StringConversion)?;

        unsafe {
            lua_setfield(self.state, idx, c_key.as_ptr());
        }

        Ok(())
    }

    /// Checks and retrieves a string from the Lua stack
    pub fn check_string(&self, idx: c_int) -> Result<String> {
        lua_check_string(self.state, idx)
    }
}

/// Concatenates two strings using Neovim's string concatenation function
fn concat_strings(s1: &str, s2: &str) -> Result<String> {
    let c_s1 = CString::new(s1).map_err(|_| Error::StringConversion)?;
    let c_s2 = CString::new(s2).map_err(|_| Error::StringConversion)?;

    unsafe {
        let result = concat_str(c_s1.as_ptr(), c_s2.as_ptr());
        let neovim_str = NeovimString::new(result);
        neovim_str.to_string()
    }
}

/// Lua function for loading a configuration file
extern "C" fn lua_load_config(l: *mut LuaState) -> c_int {
    let lua = match unsafe { Lua::new(l) } {
        Ok(lua) => lua,
        Err(_) => return 0,
    };

    let path = match lua.check_string(1) {
        Ok(path) => path,
        Err(_) => return 0,
    };

    let cmd = format!("luafile {path}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Lua function for setting Neovim options
extern "C" fn lua_opt(l: *mut LuaState) -> c_int {
    let lua = match unsafe { Lua::new(l) } {
        Ok(lua) => lua,
        Err(_) => return 0,
    };

    let key = match lua.check_string(1) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let old_val = match lua.check_string(2) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let new_val = match lua.check_string(3) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let temp = match concat_strings(&old_val, ",") {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let combined = match concat_strings(&temp, &new_val) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("set {key}={combined}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Lua function for defining key mappings
extern "C" fn lua_map(l: *mut LuaState) -> c_int {
    let lua = match unsafe { Lua::new(l) } {
        Ok(lua) => lua,
        Err(_) => return 0,
    };

    let mode = match lua.check_string(1) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let lhs = match lua.check_string(2) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let rhs = match lua.check_string(3) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("{mode}map {lhs} {rhs}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Lua function for setting global variables
extern "C" fn lua_g(l: *mut LuaState) -> c_int {
    let lua = match unsafe { Lua::new(l) } {
        Ok(lua) => lua,
        Err(_) => return 0,
    };

    let key = match lua.check_string(1) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let val = match lua.check_string(2) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("let g:{key} = {val}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Module initialization function
///
/// This function sets up the Lua module and registers all the available functions.
///
/// # Safety
///
/// The `l` parameter must be a valid pointer to an initialized Lua state.
/// This function is called by the Lua runtime and should not be called manually.
#[no_mangle]
pub unsafe extern "C" fn luaopen_init(l: *mut LuaState) -> c_int {
    let lua = match Lua::new(l) {
        Ok(lua) => lua,
        Err(_) => return 0,
    };

    lua.create_table(0, 0);

    lua.push_cclosure(lua_load_config, 0);
    if lua.set_field(-2, "load_config").is_err() {
        return 0;
    }

    lua.push_cclosure(lua_opt, 0);
    if lua.set_field(-2, "opt").is_err() {
        return 0;
    }

    lua.push_cclosure(lua_map, 0);
    if lua.set_field(-2, "map").is_err() {
        return 0;
    }

    lua.push_cclosure(lua_g, 0);
    if lua.set_field(-2, "g").is_err() {
        return 0;
    }

    // Register the extra Lua functions
    if register_extra_lua_functions(&lua).is_err() {
        return 0;
    }

    // Register plugin manager functions
    if register_plugin_functions(&lua).is_err() {
        return 0;
    }

    // Register Neovim C interop functions
    if register_nvim_interop_functions(&lua).is_err() {
        return 0;
    }

    extern "C" fn safe_luaopen_init(l: *mut LuaState) -> c_int {
        unsafe { luaopen_init(l) }
    }

    lua.push_cclosure(safe_luaopen_init, 0);
    if lua.set_field(-2, "rns").is_err() {
        return 0;
    }

    1
}

/// Sets a Neovim option by concatenating old and new values
///
/// # Safety
///
/// All pointers must be valid, properly null-terminated C strings.
/// The caller must ensure that the strings remain valid for the duration of the call.
/// This function is intended to be called from C or Lua code via FFI.
#[no_mangle]
pub unsafe extern "C" fn opt(
    key: *const c_char,
    old_val: *const c_char,
    new_val: *const c_char,
) -> c_int {
    let key_str = match extract_c_string(key) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let old_str = match extract_c_string(old_val) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let new_str = match extract_c_string(new_val) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let temp = match concat_strings(&old_str, ",") {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let combined = match concat_strings(&temp, &new_str) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("set {key_str}={combined}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Sets up a module with the given configuration
///
/// # Safety
///
/// Both `module` and `config` must be valid, properly null-terminated C strings.
/// The caller must ensure that the strings remain valid for the duration of the call.
/// This function is intended to be called from C or Lua code via FFI.
#[no_mangle]
pub unsafe extern "C" fn require_setup(module: *const c_char, config: *const c_char) -> c_int {
    let module_str = match extract_c_string(module) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let config_str = match extract_c_string(config) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("require_setup {module_str} {config_str}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Sets up an autocommand with the given event, pattern, and command
///
/// # Safety
///
/// All pointers must be valid, properly null-terminated C strings.
/// The caller must ensure that the strings remain valid for the duration of the call.
/// This function is intended to be called from C or Lua code via FFI.
#[no_mangle]
pub unsafe extern "C" fn autocmd(
    event: *const c_char,
    pattern: *const c_char,
    command: *const c_char,
) -> c_int {
    let event_str = match extract_c_string(event) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let pattern_str = match extract_c_string(pattern) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let command_str = match extract_c_string(command) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("autocmd {event_str} {pattern_str} {command_str}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Configures an LSP server with the given JSON configuration
///
/// # Safety
///
/// Both `server` and `config_json` must be valid, properly null-terminated C strings.
/// The caller must ensure that the strings remain valid for the duration of the call.
/// This function is intended to be called from C or Lua code via FFI.
#[no_mangle]
pub unsafe extern "C" fn setup_lsp(server: *const c_char, config_json: *const c_char) -> c_int {
    let server_str = match extract_c_string(server) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let config_str = match extract_c_string(config_json) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("lua require'lspconfig'.{server_str}.setup({config_str})");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Executes arbitrary Lua code
///
/// # Safety
///
/// The `code` parameter must be a valid, properly null-terminated C string.
/// The caller must ensure that the string remains valid for the duration of the call.
/// This function is intended to be called from C or Lua code via FFI.
/// Be aware that executing arbitrary Lua code can have security implications.
#[no_mangle]
pub unsafe extern "C" fn exec_lua(code: *const c_char) -> c_int {
    let code_str = match extract_c_string(code) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!("lua {code_str}");
    match run_cmd(&cmd) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Registers additional Lua functions with the module
fn register_extra_lua_functions(lua: &Lua<'_>) -> Result<()> {
    extern "C" fn lua_autocmd(l: *mut LuaState) -> c_int {
        let lua = match unsafe { Lua::new(l) } {
            Ok(lua) => lua,
            Err(_) => return 0,
        };

        let event = match lua.check_string(1) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let pattern = match lua.check_string(2) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let command = match lua.check_string(3) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let cmd = format!("autocmd {event} {pattern} {command}");
        match run_cmd(&cmd) {
            Ok(()) => 1,
            Err(_) => 0,
        }
    }

    extern "C" fn lua_exec(l: *mut LuaState) -> c_int {
        let lua = match unsafe { Lua::new(l) } {
            Ok(lua) => lua,
            Err(_) => return 0,
        };

        let code = match lua.check_string(1) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let cmd = format!("lua {code}");
        match run_cmd(&cmd) {
            Ok(()) => 1,
            Err(_) => 0,
        }
    }

    lua.push_cclosure(lua_autocmd, 0);
    lua.set_field(-2, "autocmd")?;

    lua.push_cclosure(lua_exec, 0);
    lua.set_field(-2, "exec_lua")?;

    Ok(())
}
