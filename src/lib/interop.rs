use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::{extract_c_string, Lua, LuaState, Result};

/// Sets a boolean Neovim option
///
/// # Safety
///
/// `name` must be a valid null-terminated C string pointing to a valid option name.
#[no_mangle]
pub extern "C" fn nvim_set_option_bool(name: *const c_char, value: c_int) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => {
            let cmd = if value != 0 {
                format!("set {name_str}")
            } else {
                format!("set no{name_str}")
            };

            match crate::run_cmd(&cmd) {
                Ok(()) => 1,
                Err(_) => 0,
            }
        }
        Err(_) => 0,
    }
}

/// Sets an integer Neovim option
///
/// # Safety
///
/// `name` must be a valid null-terminated C string pointing to a valid option name.
#[no_mangle]
pub extern "C" fn nvim_set_option_int(name: *const c_char, value: c_int) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => {
            let cmd = format!("set {name_str}={value}");
            match crate::run_cmd(&cmd) {
                Ok(()) => 1,
                Err(_) => 0,
            }
        }
        Err(_) => 0,
    }
}

/// Sets a string Neovim option
///
/// # Safety
///
/// `name` and `value` must be valid null-terminated C strings.
#[no_mangle]
pub extern "C" fn nvim_set_option_string(name: *const c_char, value: *const c_char) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => match extract_c_string(value) {
            Ok(value_str) => {
                let cmd = format!("set {name_str}={value_str}");
                match crate::run_cmd(&cmd) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

/// Sets a Neovim global variable
///
/// # Safety
///
/// `name` and `value` must be valid null-terminated C strings.
#[no_mangle]
pub extern "C" fn nvim_set_global(name: *const c_char, value: *const c_char) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => match extract_c_string(value) {
            Ok(value_str) => {
                let cmd = format!("let g:{}=\"{}\"", name_str, value_str.replace('"', "\\\""));
                match crate::run_cmd(&cmd) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

/// Creates a keymap in Neovim
///
/// # Safety
///
/// `mode`, `lhs`, `rhs` must be valid null-terminated C strings.
/// `_opts` is currently unused but must be either null or a valid C string.
#[no_mangle]
pub extern "C" fn nvim_create_keymap(
    mode: *const c_char,
    lhs: *const c_char,
    rhs: *const c_char,
    _opts: *const c_char,
) -> c_int {
    match extract_c_string(mode) {
        Ok(mode_str) => match (extract_c_string(lhs), extract_c_string(rhs)) {
            (Ok(lhs_str), Ok(rhs_str)) => {
                let cmd = format!("{mode_str}map {lhs_str} {rhs_str}");
                match crate::run_cmd(&cmd) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            _ => 0,
        },
        Err(_) => 0,
    }
}

/// Creates a user command in Neovim
///
/// # Safety
///
/// `name` and `command` must be valid null-terminated C strings.
/// `_opts` is currently unused but must be either null or a valid C string.
#[no_mangle]
pub extern "C" fn nvim_create_user_command(
    name: *const c_char,
    command: *const c_char,
    _opts: *const c_char,
) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => match extract_c_string(command) {
            Ok(cmd_str) => {
                let cmd = format!("command! {name_str} {cmd_str}");
                match crate::run_cmd(&cmd) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

/// Creates an autocommand in Neovim
///
/// # Safety
///
/// `event`, `pattern`, and `command` must be valid null-terminated C strings.
/// `group` must be either null or a valid null-terminated C string.
#[no_mangle]
pub extern "C" fn nvim_create_autocmd(
    event: *const c_char,
    pattern: *const c_char,
    command: *const c_char,
    group: *const c_char,
) -> c_int {
    match extract_c_string(event) {
        Ok(event_str) => match (extract_c_string(pattern), extract_c_string(command)) {
            (Ok(pattern_str), Ok(cmd_str)) => {
                let lua_cmd = format!(
                    "vim.api.nvim_create_autocmd('{}', {{ pattern = '{}', command = '{}' {}}})",
                    event_str,
                    pattern_str,
                    cmd_str,
                    if group.is_null() {
                        String::new()
                    } else {
                        match extract_c_string(group) {
                            Ok(group_str) => format!(", group = '{group_str}' "),
                            Err(_) => String::new(),
                        }
                    }
                );

                match crate::run_cmd(&format!("lua {lua_cmd}")) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            _ => 0,
        },
        Err(_) => 0,
    }
}

/// Creates an autocommand group in Neovim
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub extern "C" fn nvim_create_augroup(name: *const c_char, clear: c_int) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => {
            let lua_cmd = format!(
                "vim.api.nvim_create_augroup('{}', {{ clear = {} }})",
                name_str,
                if clear != 0 { "true" } else { "false" }
            );

            match crate::run_cmd(&format!("lua {lua_cmd}")) {
                Ok(()) => 1,
                Err(_) => 0,
            }
        }
        Err(_) => 0,
    }
}

/// Creates an autocommand group in Neovim using Lua API
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub extern "C" fn nvim_create_augroup_lua(name: *const c_char, clear: c_int) -> c_int {
    match extract_c_string(name) {
        Ok(name_str) => {
            let lua_cmd = format!(
                "vim.api.nvim_create_augroup('{}', {{ clear = {} }})",
                name_str,
                if clear != 0 { "true" } else { "false" }
            );

            match crate::run_cmd(&format!("lua {lua_cmd}")) {
                Ok(()) => 1,
                Err(_) => 0,
            }
        }
        Err(_) => 0,
    }
}

/// Creates an autocommand in Neovim using Lua API
///
/// # Safety
///
/// `event`, `pattern`, and `command` must be valid null-terminated C strings.
/// `group` must be either null or a valid null-terminated C string.
#[no_mangle]
pub extern "C" fn nvim_create_autocmd_lua(
    event: *const c_char,
    pattern: *const c_char,
    command: *const c_char,
    group: *const c_char,
) -> c_int {
    match extract_c_string(event) {
        Ok(event_str) => match (extract_c_string(pattern), extract_c_string(command)) {
            (Ok(pattern_str), Ok(cmd_str)) => {
                let lua_cmd = format!(
                    "vim.api.nvim_create_autocmd('{}', {{ pattern = '{}', command = '{}' {}}})",
                    event_str,
                    pattern_str,
                    cmd_str,
                    if group.is_null() {
                        String::new()
                    } else {
                        match extract_c_string(group) {
                            Ok(group_str) => format!(", group = '{group_str}' "),
                            Err(_) => String::new(),
                        }
                    }
                );

                match crate::run_cmd(&format!("lua {lua_cmd}")) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            _ => 0,
        },
        Err(_) => 0,
    }
}

/// Sets a buffer-local keymap in Neovim
///
/// # Safety
///
/// `mode`, `lhs`, and `rhs` must be valid null-terminated C strings.
/// `_opts` is currently unused but must be either null or a valid C string.
/// `_buffer` is currently unused.
#[no_mangle]
pub extern "C" fn nvim_buf_set_keymap(
    _buffer: c_int,
    mode: *const c_char,
    lhs: *const c_char,
    rhs: *const c_char,
    _opts: *const c_char,
) -> c_int {
    match extract_c_string(mode) {
        Ok(mode_str) => match (extract_c_string(lhs), extract_c_string(rhs)) {
            (Ok(lhs_str), Ok(rhs_str)) => {
                let cmd = format!("{mode_str}map <buffer> {lhs_str} {rhs_str}");
                match crate::run_cmd(&cmd) {
                    Ok(()) => 1,
                    Err(_) => 0,
                }
            }
            _ => 0,
        },
        Err(_) => 0,
    }
}

/// Executes a Neovim command
///
/// # Safety
///
/// `command` must be a valid null-terminated C string containing a valid Neovim command.
#[no_mangle]
pub extern "C" fn nvim_exec_command(command: *const c_char) -> c_int {
    match extract_c_string(command) {
        Ok(cmd_str) => match crate::run_cmd(&cmd_str) {
            Ok(()) => 1,
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

/// Registers Neovim interop functions with the Lua state
pub fn register_nvim_interop_functions(lua: &Lua<'_>) -> Result<()> {
    extern "C" fn lua_nvim_set_option_bool(l: *mut LuaState) -> c_int {
        let lua = match unsafe { Lua::new(l) } {
            Ok(lua) => lua,
            Err(_) => return 0,
        };

        let name = match lua.check_string(1) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let value = unsafe { lua_toboolean(l, 2) };

        nvim_set_option_bool(CString::new(name).unwrap().as_ptr(), value)
    }

    extern "C" fn lua_nvim_create_keymap(l: *mut LuaState) -> c_int {
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

        let opts = lua.check_string(4).unwrap_or_default();

        let opts_ptr = if opts.is_empty() {
            std::ptr::null()
        } else {
            CString::new(opts).unwrap().as_ptr()
        };

        nvim_create_keymap(
            CString::new(mode).unwrap().as_ptr(),
            CString::new(lhs).unwrap().as_ptr(),
            CString::new(rhs).unwrap().as_ptr(),
            opts_ptr,
        )
    }

    lua.push_cclosure(lua_nvim_set_option_bool, 0);
    lua.set_field(-2, "set_option_bool")?;

    lua.push_cclosure(lua_nvim_create_keymap, 0);
    lua.set_field(-2, "create_keymap")?;

    Ok(())
}

// Lua API bindings used in the functions
extern "C" {
    /// Converts a Lua value at the given index to a boolean
    ///
    /// # Safety
    ///
    /// `l` must be a valid pointer to a properly initialized Lua state.
    /// The index must be valid (not beyond the stack size).
    fn lua_toboolean(l: *mut LuaState, idx: c_int) -> c_int;
}
