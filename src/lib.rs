use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct LuaState {
    _private: [u8; 0],
}

// FFI bindings to the Lua C API
extern "C" {
    // Creates a new table and pushes it onto the stack.
    fn lua_createtable(L: *mut LuaState, narr: c_int, nrec: c_int);

    // Pushes a C closure onto the stack
    fn lua_pushcclosure(L: *mut LuaState, f: extern "C" fn(*mut LuaState) -> c_int, n: c_int);

    // Sets the field t[k] with the value on the top of the stack
    fn lua_setfield(L: *mut LuaState, idx: c_int, k: *const c_char);

    // Checks that the value at the given stack index is a string and returns it
    fn luaL_checklstring(L: *mut LuaState, arg: c_int, len: *mut usize) -> *const c_char;
}

// FFI bindings to the external Neovim API
extern "C" {
    /// Executes a command
    pub fn do_cmdline_cmd(cmd: *const c_char) -> c_int;

    /// Concatenates two C strings; the returned pointer is managed by Neovim
    pub fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
}

/// Helper function to extract a string from Lua at the given stack index
fn lua_check_string(L: *mut LuaState, idx: c_int) -> Option<String> {
    unsafe {
        let mut len: usize = 0;
        let ptr = luaL_checklstring(L, idx, &mut len);
        if ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }
}

/// Run a command by converting the Rust &str to a C string
fn run_cmd(cmd: &str) -> bool {
    let c_cmd = match CString::new(cmd) {
        Ok(c_cmd) => c_cmd,
        Err(_) => return false,
    };

    // Only this FFI call is unsafe.
    let result = unsafe { do_cmdline_cmd(c_cmd.as_ptr()) };

    result == 0
}

/// A safe wrapper around a Lua state pointer
///
/// # Safety
///
/// The caller must guarantee that `state` is non-null and remains valid for the lifetime
/// of this wrapper
pub struct Lua<'a> {
    state: *mut LuaState,
    _marker: PhantomData<&'a LuaState>,
}

impl Lua<'_> {
    pub unsafe fn new(state: *mut LuaState) -> Self {
        assert!(!state.is_null());
        Lua {
            state,
            _marker: PhantomData,
        }
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

    /// Sets a field in the Lua table at the given stack index
    pub fn set_field(&self, idx: c_int, k: &str) -> bool {
        let c_key = match CString::new(k) {
            Ok(c_key) => c_key,
            Err(_) => return false,
        };

        unsafe {
            lua_setfield(self.state, idx, c_key.as_ptr());
        }

        true
    }

    /// Gets a string argument from the Lua stack
    pub fn check_string(&self, idx: c_int) -> Option<String> {
        lua_check_string(self.state, idx)
    }
}

/// Lua binding for loading an external Lua configuration file
/// Usage in Lua: `rns.load_config("path/to/config.lua`")
extern "C" fn lua_load_config(L: *mut LuaState) -> c_int {
    // Extract path string
    let path = match lua_check_string(L, 1) {
        Some(path) => path,
        None => return 0,
    };

    let cmd = format!("luafile {path}");
    if run_cmd(&cmd) {
        1
    } else {
        0
    }
}

/// Lua binding for setting an option
/// Usage in Lua: `rns.opt("option_name`", "`old_value`", "`new_value`")
extern "C" fn lua_opt(L: *mut LuaState) -> c_int {
    // Extract string arguments
    let key = match lua_check_string(L, 1) {
        Some(s) => s,
        None => return 0,
    };

    let old_val = match lua_check_string(L, 2) {
        Some(s) => s,
        None => return 0,
    };

    let new_val = match lua_check_string(L, 3) {
        Some(s) => s,
        None => return 0,
    };

    // Concatenate using the external API.
    let comma = CString::new(",").unwrap();
    let combined = unsafe {
        let temp = concat_str(CString::new(old_val).unwrap().as_ptr(), comma.as_ptr());
        let result = concat_str(temp, CString::new(new_val).unwrap().as_ptr());
        CStr::from_ptr(result).to_string_lossy().into_owned()
    };

    let cmd = format!("set {key}={combined}");
    if run_cmd(&cmd) {
        1
    } else {
        0
    }
}

/// Lua binding for setting a key mapping
/// Usage in Lua: rns.map("n", "<leader>x", ":echo 'Hello'<CR>")
extern "C" fn lua_map(L: *mut LuaState) -> c_int {
    // Extract string arguments
    let mode = match lua_check_string(L, 1) {
        Some(s) => s,
        None => return 0,
    };

    let lhs = match lua_check_string(L, 2) {
        Some(s) => s,
        None => return 0,
    };

    let rhs = match lua_check_string(L, 3) {
        Some(s) => s,
        None => return 0,
    };

    let cmd = format!("{mode}map {lhs} {rhs}");
    if run_cmd(&cmd) {
        1
    } else {
        0
    }
}

/// Refactored Lua binding for setting a global variable
/// Usage in Lua: `rns.g("my_var`", "value")
extern "C" fn lua_g(L: *mut LuaState) -> c_int {
    // Extract string arguments
    let key = match lua_check_string(L, 1) {
        Some(s) => s,
        None => return 0,
    };

    let val = match lua_check_string(L, 2) {
        Some(s) => s,
        None => return 0,
    };

    let cmd = format!("let g:{key} = {val}");
    if run_cmd(&cmd) {
        1
    } else {
        0
    }
}

/// Module initialization
#[no_mangle]
pub unsafe extern "C" fn luaopen_init(L: *mut LuaState) -> c_int {
    let lua = Lua::new(L);
    lua.create_table(0, 0);

    // Register functions
    lua.push_cclosure(lua_load_config, 0);
    if !lua.set_field(-2, "load_config") {
        return 0;
    }

    lua.push_cclosure(lua_opt, 0);
    if !lua.set_field(-2, "opt") {
        return 0;
    }

    lua.push_cclosure(lua_map, 0);
    if !lua.set_field(-2, "map") {
        return 0;
    }

    lua.push_cclosure(lua_g, 0);
    if !lua.set_field(-2, "g") {
        return 0;
    }

    extern "C" fn safe_luaopen_init(L: *mut LuaState) -> c_int {
        unsafe { luaopen_init(L) }
    }

    lua.push_cclosure(safe_luaopen_init, 0);
    if !lua.set_field(-2, "rns") {
        return 0;
    }

    1 // Return 1 to indicate success
}

#[no_mangle]
pub extern "C" fn opt(key: *const c_char, old_val: *const c_char, new_val: *const c_char) {
    // Convert the C strings, build commands, etc.
    unsafe {
        let key = if key.is_null() {
            return;
        } else {
            CStr::from_ptr(key).to_string_lossy()
        };
        let old = if old_val.is_null() {
            return;
        } else {
            CStr::from_ptr(old_val).to_string_lossy()
        };
        let new = if new_val.is_null() {
            return;
        } else {
            CStr::from_ptr(new_val).to_string_lossy()
        };

        let comma = CString::new(",").unwrap();
        let temp = concat_str(old.as_ptr().cast::<c_char>(), comma.as_ptr());
        let combined = concat_str(temp, new.as_ptr().cast::<c_char>());
        let combined_str = CStr::from_ptr(combined).to_string_lossy();
        let cmd = format!("set {key}={combined_str}");
        let _ = run_cmd(&cmd);
    }
}

#[no_mangle]
pub extern "C" fn require_setup(module: *const c_char, config: *const c_char) {
    unsafe {
        if module.is_null() || config.is_null() {
            return;
        }
        let module_str = CStr::from_ptr(module).to_string_lossy();
        let config_str = CStr::from_ptr(config).to_string_lossy();
        let cmd = format!("require_setup {module_str} {config_str}");
        let _ = run_cmd(&cmd);
    }
}
