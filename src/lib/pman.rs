use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::extract_c_string;

// Plugin configuration state
static mut CURRENT_PLUGIN: Option<String> = None;
static mut PLUGIN_CONFIG: Option<String> = None;

/// Registers a plugin with the plugin manager
///
/// # Safety
///
/// `name` and `url` must be valid null-terminated C strings.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn register_plugin(name: *const c_char, url: *const c_char) -> c_int {
    let name_str = match extract_c_string(name) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let url_str = match extract_c_string(url) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!(
        "if not _G.plugins then _G.plugins = {{}} end;\
         _G.plugins['{name_str}'] = {{ url = '{url_str}', enabled = true }}"
    );

    match crate::run_cmd(&format!("lua {cmd}")) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Applies configuration to a plugin
///
/// # Safety
///
/// `name` and `config` must be valid null-terminated C strings.
/// This function passes the configuration directly to Lua for execution.
#[no_mangle]
pub unsafe extern "C" fn configure_plugin(name: *const c_char, config: *const c_char) -> c_int {
    let name_str = match extract_c_string(name) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let config_str = match extract_c_string(config) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let cmd = format!(
        "if _G.plugins and _G.plugins['{name_str}'] then _G.plugins['{name_str}'].config = [===[{config_str}]===] end"
    );

    match crate::run_cmd(&format!("lua {cmd}")) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Begins configuration for a plugin
///
/// # Safety
///
/// `plugin_name` must be a valid null-terminated C string.
/// This function modifies static mutable state and must not be called concurrently.
/// Must be paired with a matching call to `plugin_config_end`.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_begin(plugin_name: *const c_char) -> c_int {
    match extract_c_string(plugin_name) {
        Ok(name) => {
            CURRENT_PLUGIN = Some(name);
            PLUGIN_CONFIG = Some(String::new());
            1
        }
        Err(_) => 0,
    }
}

/// Finalizes and applies plugin configuration
///
/// # Safety
///
/// Must be called only after a successful call to `plugin_config_begin`.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_end() -> c_int {
    if let (Some(plugin), Some(config)) = (&CURRENT_PLUGIN, &PLUGIN_CONFIG) {
        let result = configure_plugin(
            CString::new(plugin.as_str()).unwrap().as_ptr(),
            CString::new(config.as_str()).unwrap().as_ptr(),
        );

        CURRENT_PLUGIN = None;
        PLUGIN_CONFIG = None;

        result
    } else {
        0
    }
}

/// Adds an LSP server to the current plugin configuration
///
/// # Safety
///
/// `server_name` must be a valid null-terminated C string.
/// Must be called between `plugin_config_begin` and `plugin_config_end`.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_add_server(server_name: *const c_char) -> c_int {
    match extract_c_string(server_name) {
        Ok(server) => {
            if let Some(config) = &mut PLUGIN_CONFIG {
                config.push_str(&format!("require('lspconfig').{server}.setup({{}});\n"));
                1
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

/// Sets an option for an LSP server in the current plugin configuration
///
/// # Safety
///
/// `server`, `option`, and `value` must be valid null-terminated C strings.
/// Must be called between `plugin_config_begin` and `plugin_config_end`.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_set_server_option(
    server: *const c_char,
    option: *const c_char,
    value: *const c_char,
) -> c_int {
    match (
        extract_c_string(server),
        extract_c_string(option),
        extract_c_string(value),
    ) {
        (Ok(server_str), Ok(option_str), Ok(value_str)) => {
            if let Some(config) = &mut PLUGIN_CONFIG {
                config.push_str(&format!(
                    "require('lspconfig').{server_str}.setup({{ settings = {{ ['{server_str}'] = {{ {option_str} = '{value_str}' }} }} }});\n"
                ));
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Sets a key mapping for a plugin in the current plugin configuration
///
/// # Safety
///
/// `plugin`, `mode`, `key`, and `action` must be valid null-terminated C strings.
/// Must be called between `plugin_config_begin` and `plugin_config_end`.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_set_mapping(
    plugin: *const c_char,
    mode: *const c_char,
    key: *const c_char,
    action: *const c_char,
) -> c_int {
    match (
        extract_c_string(plugin),
        extract_c_string(mode),
        extract_c_string(key),
        extract_c_string(action),
    ) {
        (Ok(plugin_str), Ok(mode_str), Ok(key_str), Ok(action_str)) => {
            if let Some(config) = &mut PLUGIN_CONFIG {
                config.push_str(&format!(
                    "require('{plugin_str}').setup({{ defaults = {{ mappings = {{ {mode_str} = {{ ['{key_str}'] = '{action_str}' }} }} }} }});\n"
                ));
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Adds a telescope keymap to the current plugin configuration
///
/// # Safety
///
/// `mode`, `key`, and `command` must be valid null-terminated C strings.
/// `_plugin` is currently unused but must be a valid pointer.
/// Must be called between `plugin_config_begin` and `plugin_config_end`.
/// This function modifies static mutable state and must not be called concurrently.
#[no_mangle]
pub unsafe extern "C" fn plugin_config_add_keymap(
    mode: *const c_char,
    key: *const c_char,
    _plugin: *const c_char,
    command: *const c_char,
) -> c_int {
    match (
        extract_c_string(mode),
        extract_c_string(key),
        extract_c_string(command),
    ) {
        (Ok(mode_str), Ok(key_str), Ok(cmd_str)) => {
            if let Some(config) = &mut PLUGIN_CONFIG {
                config.push_str(&format!(
                    "vim.keymap.set('{mode_str}', '{key_str}', '<cmd>Telescope {cmd_str}<CR>');\n"
                ));
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Installs all registered plugins
///
/// # Safety
///
/// This function executes Lua code that interacts with the filesystem.
/// It should be called when Neovim is ready to load plugins.
#[no_mangle]
pub unsafe extern "C" fn install_plugins() -> c_int {
    let cmd = r"
        if not _G.plugins then return end
        local data_dir = vim.fn.stdpath('data')
        local plugin_dir = data_dir .. '/site/pack/managed/start/'

        if vim.fn.isdirectory(plugin_dir) == 0 then
            vim.fn.mkdir(plugin_dir, 'p')
        end

        for name, plugin in pairs(_G.plugins) do
            if plugin.enabled then
                local plugin_path = plugin_dir .. name
                if vim.fn.isdirectory(plugin_path) == 0 then
                    vim.notify('Installing ' .. name .. '...')
                    vim.fn.system({'git', 'clone', '--depth', '1', plugin.url, plugin_path})
                end
                plugin.path = plugin_path
                vim.opt.rtp:prepend(plugin_path)
            end
        end

        vim.cmd('packloadall')
        vim.cmd('runtime! plugin/**/*.vim plugin/**/*.lua')
        vim.cmd('silent! helptags ALL')
    ";

    match crate::run_cmd(&format!("lua {cmd}")) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Loads configurations for all registered plugins
///
/// # Safety
///
/// This function evaluates arbitrary Lua code stored in plugin configurations.
/// It should be called after plugins are installed and Neovim is fully initialized.
#[no_mangle]
pub unsafe extern "C" fn load_plugin_configs() -> c_int {
    let cmd = r#"
        if not _G.plugins then return end
        for name, plugin in pairs(_G.plugins) do
            if plugin.enabled and plugin.config then
                local success, err = pcall(function()
                    local status, mod = pcall(require, name)
                    if status then
                        local chunk, err = loadstring(plugin.config)
                        if chunk then
                            chunk()
                        else
                            error("Failed to parse configuration: " .. err)
                        end
                    else
                        error("Module not found")
                    end
                end)

                if not success then
                    vim.schedule(function()
                        local retry, rerr = pcall(function()
                            local status, mod = pcall(require, name)
                            if status then
                                local chunk = loadstring(plugin.config)
                                if chunk then
                                    chunk()
                                end
                            end
                        end)

                        if not retry then
                            vim.notify('Cannot configure ' .. name .. ': ' .. tostring(err), vim.log.levels.WARN)
                        end
                    end)
                end
            end
        end
    "#;

    match crate::run_cmd(&format!("lua {cmd}")) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Updates all registered plugins using git
///
/// # Safety
///
/// This function executes system commands that interact with the filesystem.
/// It should be called when Neovim is ready to update plugins.
#[no_mangle]
pub unsafe extern "C" fn update_plugins() -> c_int {
    let cmd = r"
        if not _G.plugins then return end
        local data_dir = vim.fn.stdpath('data')
        local plugin_dir = data_dir .. '/site/pack/managed/start/'

        for name, plugin in pairs(_G.plugins) do
            if plugin.enabled then
                local plugin_path = plugin_dir .. name
                if vim.fn.isdirectory(plugin_path) == 1 then
                    vim.notify('Updating ' .. name)
                    vim.fn.system({'git', '-C', plugin_path, 'pull', '--ff-only'})
                end
            end
        end

        vim.cmd('packloadall')
        vim.cmd('runtime! plugin/**/*.vim plugin/**/*.lua')
        vim.cmd('silent! helptags ALL')
    ";

    match crate::run_cmd(&format!("lua {cmd}")) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Registers Lua bindings for plugin management functions
pub fn register_plugin_functions(lua: &crate::Lua<'_>) -> crate::Result<()> {
    extern "C" fn lua_register_plugin(l: *mut crate::LuaState) -> c_int {
        let lua = match unsafe { crate::Lua::new(l) } {
            Ok(lua) => lua,
            Err(_) => return 0,
        };

        let name = match lua.check_string(1) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let url = match lua.check_string(2) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        unsafe {
            register_plugin(
                CString::new(name).unwrap().as_ptr(),
                CString::new(url).unwrap().as_ptr(),
            )
        }
    }

    extern "C" fn lua_configure_plugin(l: *mut crate::LuaState) -> c_int {
        let lua = match unsafe { crate::Lua::new(l) } {
            Ok(lua) => lua,
            Err(_) => return 0,
        };

        let name = match lua.check_string(1) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let config = match lua.check_string(2) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        unsafe {
            configure_plugin(
                CString::new(name).unwrap().as_ptr(),
                CString::new(config).unwrap().as_ptr(),
            )
        }
    }

    extern "C" fn lua_install_plugins(_l: *mut crate::LuaState) -> c_int {
        unsafe { install_plugins() }
    }

    extern "C" fn lua_update_plugins(_l: *mut crate::LuaState) -> c_int {
        unsafe { update_plugins() }
    }

    extern "C" fn lua_load_plugin_configs(_l: *mut crate::LuaState) -> c_int {
        unsafe { load_plugin_configs() }
    }

    lua.push_cclosure(lua_register_plugin, 0);
    lua.set_field(-2, "register_plugin")?;

    lua.push_cclosure(lua_configure_plugin, 0);
    lua.set_field(-2, "configure_plugin")?;

    lua.push_cclosure(lua_install_plugins, 0);
    lua.set_field(-2, "install_plugins")?;

    lua.push_cclosure(lua_update_plugins, 0);
    lua.set_field(-2, "update_plugins")?;

    lua.push_cclosure(lua_load_plugin_configs, 0);
    lua.set_field(-2, "load_configs")?;

    Ok(())
}
