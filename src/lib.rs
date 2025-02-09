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

/// Run a command by converting the Rust &str to a C string
fn run_cmd(cmd: &str) {
    let c_cmd = CString::new(cmd).unwrap();
    // Only this FFI call is unsafe.
    unsafe {
        do_cmdline_cmd(c_cmd.as_ptr());
    }
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

impl<'a> Lua<'a> {
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
    pub fn set_field(&self, idx: c_int, k: &CStr) {
        unsafe {
            lua_setfield(self.state, idx, k.as_ptr());
        }
    }
}

/// Lua binding for loading an external Lua configuration file
/// Usage in Lua: rns.load_config("path/to/config.lua")
extern "C" fn lua_load_config(L: *mut LuaState) -> c_int {
    // Isolate the unsafe FFI call to extract the string argument.
    let path = {
        unsafe {
            let mut len: usize = 0;
            let ptr = luaL_checklstring(L, 1, &mut len);
            if ptr.is_null() {
                return 0;
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    };

    let cmd = format!("luafile {}", path);
    run_cmd(&cmd);
    0
}

/// Lua binding for setting an option
/// Usage in Lua: rns.opt("option_name", "old_value", "new_value")
extern "C" fn lua_opt(L: *mut LuaState) -> c_int {
    let (key, old_val, new_val) = unsafe {
        let mut len1 = 0;
        let mut len2 = 0;
        let mut len3 = 0;
        let key_ptr = luaL_checklstring(L, 1, &mut len1);
        let old_ptr = luaL_checklstring(L, 2, &mut len2);
        let new_ptr = luaL_checklstring(L, 3, &mut len3);
        if key_ptr.is_null() || old_ptr.is_null() || new_ptr.is_null() {
            return 0;
        }
        (
            CStr::from_ptr(key_ptr).to_string_lossy().into_owned(),
            CStr::from_ptr(old_ptr).to_string_lossy().into_owned(),
            CStr::from_ptr(new_ptr).to_string_lossy().into_owned(),
        )
    };

    // Concatenate using the external API.
    let comma = CString::new(",").unwrap();
    let combined = unsafe {
        let temp = concat_str(CString::new(old_val).unwrap().as_ptr(), comma.as_ptr());
        let result = concat_str(temp, CString::new(new_val).unwrap().as_ptr());
        CStr::from_ptr(result).to_string_lossy().into_owned()
    };

    let cmd = format!("set {}={}", key, combined);
    run_cmd(&cmd);
    0
}

/// Lua binding for setting a key mapping
/// Usage in Lua: rns.map("n", "<leader>x", ":echo 'Hello'<CR>")
extern "C" fn lua_map(L: *mut LuaState) -> c_int {
    let (mode, lhs, rhs) = unsafe {
        let mut len1 = 0;
        let mut len2 = 0;
        let mut len3 = 0;
        let mode_ptr = luaL_checklstring(L, 1, &mut len1);
        let lhs_ptr = luaL_checklstring(L, 2, &mut len2);
        let rhs_ptr = luaL_checklstring(L, 3, &mut len3);
        if mode_ptr.is_null() || lhs_ptr.is_null() || rhs_ptr.is_null() {
            return 0;
        }
        (
            CStr::from_ptr(mode_ptr).to_string_lossy().into_owned(),
            CStr::from_ptr(lhs_ptr).to_string_lossy().into_owned(),
            CStr::from_ptr(rhs_ptr).to_string_lossy().into_owned(),
        )
    };

    let cmd = format!("{}map {} {}", mode, lhs, rhs);
    run_cmd(&cmd);
    0
}

/// Refactored Lua binding for setting a global variable
/// Usage in Lua: rns.g("my_var", "value")
extern "C" fn lua_g(L: *mut LuaState) -> c_int {
    let (key, val) = unsafe {
        let mut len1 = 0;
        let mut len2 = 0;
        let key_ptr = luaL_checklstring(L, 1, &mut len1);
        let val_ptr = luaL_checklstring(L, 2, &mut len2);
        if key_ptr.is_null() || val_ptr.is_null() {
            return 0;
        }
        (
            CStr::from_ptr(key_ptr).to_string_lossy().into_owned(),
            CStr::from_ptr(val_ptr).to_string_lossy().into_owned(),
        )
    };

    let cmd = format!("let g:{} = {}", key, val);
    run_cmd(&cmd);
    0
}

/// Module initialization
#[no_mangle]
pub unsafe extern "C" fn luaopen_init(L: *mut LuaState) -> c_int {
    // Constructing the wrapper is unsafe. After that, our methods *should* be safe?
    // I think..?
    let lua = Lua::new(L);
    lua.create_table(0, 0);

    // Register functions
    let load_config_name = CString::new("load_config").unwrap();
    lua.push_cclosure(lua_load_config, 0);
    lua.set_field(-2, &load_config_name);

    let opt_name = CString::new("opt").unwrap();
    lua.push_cclosure(lua_opt, 0);
    lua.set_field(-2, &opt_name);

    let map_name = CString::new("map").unwrap();
    lua.push_cclosure(lua_map, 0);
    lua.set_field(-2, &map_name);

    let g_name = CString::new("g").unwrap();
    lua.push_cclosure(lua_g, 0);
    lua.set_field(-2, &g_name);

    // Oh my fucking god lmao.
    // You see, we need a safe wrapper for the unsafe code. This looks
    // *hella* unsafe but the compiler is okay with it, and so am I.
    extern "C" fn safe_luaopen_init(L: *mut LuaState) -> c_int {
        unsafe { luaopen_init(L) }
    }

    let rns_name = CString::new("rns").unwrap();
    lua.push_cclosure(safe_luaopen_init, 0);
    lua.set_field(-2, &rns_name);

    return 1; // Return 1 to indicate success
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
        let temp = concat_str(old.as_ptr() as *const c_char, comma.as_ptr());
        let combined = concat_str(temp, new.as_ptr() as *const c_char);
        let combined_str = CStr::from_ptr(combined).to_string_lossy();
        let cmd = format!("set {}={}", key, combined_str);
        run_cmd(&cmd);
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
        let cmd = format!("require_setup {} {}", module_str, config_str);
        run_cmd(&cmd);
    }
}
