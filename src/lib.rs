#![allow(non_snake_case)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct LuaState {
    _private: [u8; 0],
}

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

// External Neovim API functions
extern "C" {
    /// Executes a command
    pub fn do_cmdline_cmd(cmd: *const c_char) -> c_int;

    /// Concatenates two C strings; the returned pointer is managed by Neovim
    pub fn concat_str(s1: *const c_char, s2: *const c_char) -> *mut c_char;
}

unsafe fn run_cmd(cmd: &str) {
    let cstr = CString::new(cmd).unwrap();
    do_cmdline_cmd(cstr.as_ptr());
}

/// Lua binding for loading an external Lua configuration file.
/// Usage in Lua: nrs.load_config("path/to/config.lua")
extern "C" fn lua_load_config(L: *mut LuaState) -> c_int {
    unsafe {
        let mut len: usize = 0;
        let path_ptr = luaL_checklstring(L, 1, &mut len);
        if path_ptr.is_null() {
            return 0;
        }
        let path = CStr::from_ptr(path_ptr).to_string_lossy();
        let cmd = format!("luafile {}", path);
        run_cmd(&cmd);
    }
    0
}

/// Lua binding for setting an option
/// Usage in Lua: nrs.opt("option_name", "old_value", "new_value")
extern "C" fn lua_opt(L: *mut LuaState) -> c_int {
    unsafe {
        let mut len1: usize = 0;
        let mut len2: usize = 0;
        let mut len3: usize = 0;
        let key_ptr = luaL_checklstring(L, 1, &mut len1);
        let old_ptr = luaL_checklstring(L, 2, &mut len2);
        let new_ptr = luaL_checklstring(L, 3, &mut len3);
        if key_ptr.is_null() || old_ptr.is_null() || new_ptr.is_null() {
            return 0;
        }
        // Use concat_str to combine the old and new values (with a comma separator)
        let comma = CString::new(",").unwrap();
        let temp = concat_str(old_ptr, comma.as_ptr());
        let combined = concat_str(temp, new_ptr);
        let combined_str = CStr::from_ptr(combined).to_string_lossy();
        let key = CStr::from_ptr(key_ptr).to_string_lossy();
        let cmd = format!("set {}={}", key, combined_str);
        run_cmd(&cmd);
    }
    0
}

/// Lua binding for setting a key mapping
/// Usage in Lua: nrs.map("n", "<leader>x", ":echo 'Hello'<CR>")
extern "C" fn lua_map(L: *mut LuaState) -> c_int {
    unsafe {
        let mut len1: usize = 0;
        let mut len2: usize = 0;
        let mut len3: usize = 0;
        let mode_ptr = luaL_checklstring(L, 1, &mut len1);
        let lhs_ptr = luaL_checklstring(L, 2, &mut len2);
        let rhs_ptr = luaL_checklstring(L, 3, &mut len3);
        if mode_ptr.is_null() || lhs_ptr.is_null() || rhs_ptr.is_null() {
            return 0;
        }
        let mode = CStr::from_ptr(mode_ptr).to_string_lossy();
        let lhs = CStr::from_ptr(lhs_ptr).to_string_lossy();
        let rhs = CStr::from_ptr(rhs_ptr).to_string_lossy();
        let cmd = format!("{}map {} {}", mode, lhs, rhs);
        run_cmd(&cmd);
    }
    0
}

/// Lua binding for setting a global variable
/// Usage in Lua: nrs.g("my_var", "value")
extern "C" fn lua_g(L: *mut LuaState) -> c_int {
    unsafe {
        let mut len1: usize = 0;
        let mut len2: usize = 0;
        let key_ptr = luaL_checklstring(L, 1, &mut len1);
        let val_ptr = luaL_checklstring(L, 2, &mut len2);

        if key_ptr.is_null() || val_ptr.is_null() {
            return 0;
        }
        let key = CStr::from_ptr(key_ptr).to_string_lossy();
        let val = CStr::from_ptr(val_ptr).to_string_lossy();
        let cmd = format!("let g:{} = {}", key, val);
        run_cmd(&cmd);
    }
    0
}

/// Module initialization function
#[no_mangle]
pub unsafe extern "C" fn luaopen_init(L: *mut LuaState) -> c_int {
    // Create a new table (using lua_createtable(L, 0, 0)).
    lua_createtable(L, 0, 0);

    // Register "load_config"
    let load_config_name = CString::new("load_config").unwrap();
    lua_pushcclosure(L, lua_load_config, 0);
    lua_setfield(L, -2, load_config_name.as_ptr());

    // Register "opt"
    let opt_name = CString::new("opt").unwrap();
    lua_pushcclosure(L, lua_opt, 0);
    lua_setfield(L, -2, opt_name.as_ptr());

    // Register "map"
    let map_name = CString::new("map").unwrap();
    lua_pushcclosure(L, lua_map, 0);
    lua_setfield(L, -2, map_name.as_ptr());

    // Register "g"
    let g_name = CString::new("g").unwrap();
    lua_pushcclosure(L, lua_g, 0);
    lua_setfield(L, -2, g_name.as_ptr());

    // The module table is on top of the stack.
    // Return 1 to indicate one return value.
    1
}
