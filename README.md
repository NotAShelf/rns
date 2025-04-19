# RNS

A library for writing Neovim configurations in systems programming languages (C,
Zig, or Rust). RNS provides a consistent C-compatible FFI layer that lets you
craft your Neovim setup using compiled languages instead of Lua. _Highly_
experimental, almost fully untested. If you look at this and go "wow what a
great idea, I will start using this project right this instance!" I have
unfortunate news for your friends and family and it involves you being sent to a
mental hospital.

Greatly inspired by the awesome [CatNvim](https://github.com/rewhile/CatNvim),
redesigned as a more generic and extensible

## Features

- Write Neovim configurations in C, Zig, or Rust
- Plugin management API (register, install, update)
- Structured plugin configuration system
- LSP server configuration helpers
- Keymapping creation with a clean API
- Autocommand and autogroup management
- Option setting with appropriate type enforcement

## Usage

> [!WARNING]
> The API might change at any given time and without _any_ notice. This is not
> intended for public use (or any use, really) and I will not make any attempts
> to keep the library or its API stable.

Build RNS as a shared library:

```bash
# This will create target/release/librns.so
cargo build --release
```

Move it to your working directory, and you may begin interfacing with the
resulting shared library in C.

```bash
# Copy to working directory and create include directory
# You could also symlink it if you plan to hack on RNS.
cp target/release/librns.so ./librns.so
mkdir -p include
cp rns.h include/
```

Write your configuration in C, and then compile it with `librns` in your library
path. Note that you will also need `librns.so` in Neovim's `LD_LIBRARY_PATH`
later on.

```bash
gcc -o config.so -shared -fPIC init.c -Ltarget/release -lrns -Wl,-rpath,./ -I./include
```

Now you can load your new, compiled configuration:

```lua
-- init.lua
package.cpath = package.cpath .. ";./?.so"
local config = require("config")
```

```bash
# Ensure that librns.so is in Neovim's library path before
# you run this. Otherwise your configuration will not be
# loaded. Modifying cpath is a way of doing this but you
# might also want to set it manually.
nvim --clean -u path/to/your/init.lua
```

Now you may evaluate your life choices and consider how you even got here!
Enjoy.

### C

This is an example in C, just to give you an idea. I haven't done much to make
installation or consumption of this library ergonomic, but you are welcome to
submit pull requests or open issues to discuss your ideas :)

Just to give you an idea of the truly awesome API. Please consider writing your
own configurations if you end up using this :)

```c
#include "include/rns.h"
#include <lauxlib.h>
#include <lua.h>
#include <lualib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern int luaopen_init(lua_State *L);

static int set_config(lua_State *L) {
  nvim_set_global("mapleader", " ");
  nvim_set_global("maplocalleader", ",");
  nvim_set_option_bool("number", 1);
  nvim_set_option_bool("relativenumber", 1);
  nvim_set_option_bool("expandtab", 1);
  nvim_set_option_int("tabstop", 2);
  nvim_set_option_int("shiftwidth", 2);
  nvim_set_option_bool("cursorline", 1);
  nvim_set_option_string("signcolumn", "yes");

  nvim_exec_command("colorscheme habamax");
  return 0;
}

static int setup_keymaps(lua_State *L) {
  nvim_create_keymap("n", "<leader>w", ":w<CR>", NULL);
  nvim_create_keymap("n", "<leader>q", ":q<CR>", NULL);
  nvim_create_keymap("n", "<C-h>", "<C-w>h", NULL);
  nvim_create_keymap("n", "<C-j>", "<C-w>j", NULL);
  nvim_create_keymap("n", "<C-k>", "<C-w>k", NULL);
  nvim_create_keymap("n", "<C-l>", "<C-w>l", NULL);
  return 0;
}

static int setup_autocmds(lua_State *L) {
  nvim_create_augroup_lua("MySettings", 1);
  nvim_create_autocmd_lua("FileType", "markdown,text",
                          "setlocal wrap linebreak", "MySettings");
  nvim_create_autocmd_lua("FileType", "rust", "setlocal tabstop=4 shiftwidth=4",
                          "MySettings");
  return 0;
}

static int register_plugins(lua_State *L) {
  register_plugin("nvim-lspconfig", "https://github.com/neovim/nvim-lspconfig");
  register_plugin("telescope",
                  "https://github.com/nvim-telescope/telescope.nvim");
  register_plugin("plenary", "https://github.com/nvim-lua/plenary.nvim");
  return 0;
}

static int setup_plugin_install(lua_State *L) {
  install_plugins();
  return 0;
}

static int configure_plugins(lua_State *L) {
  // Configure LSP in a structured way
  plugin_config_begin("nvim-lspconfig");
  plugin_config_add_server("rust_analyzer");
  plugin_config_set_server_option("rust_analyzer", "checkOnSave.command",
                                  "clippy");
  plugin_config_end();

  // Configure telescope in a structured way
  plugin_config_begin("telescope");
  plugin_config_set_mapping("telescope", "i", "<C-j>", "move_selection_next");
  plugin_config_add_keymap("n", "<leader>ff", "telescope", "find_files");
  plugin_config_end();

  // Load all plugin configurations
  load_plugin_configs();

  return 0;
}

__attribute__((visibility("default"))) int luaopen_config(lua_State *L) {
  luaopen_init(L);

  luaL_Reg funcs[] = {{"set_options", set_config},
                      {"setup_keymaps", setup_keymaps},
                      {"setup_autocmds", setup_autocmds},
                      {"register_plugins", register_plugins},
                      {"install_plugins", setup_plugin_install},
                      {"configure_plugins", configure_plugins},
                      {NULL, NULL}};

  luaL_newlib(L, funcs);
  return 1;
}
```

**compile your configuration** and then load it in your `init.lua`:

```lua
-- Basic setup
package.cpath = package.cpath .. ";./?.so"

-- Load the config module
local config = require("config")

-- Print debug info
print("Configuration process completed")
```

If everything went well, Neovim will load without errors.

### For Nix Users

A basic flake.nix is provided by this repository. It is the only supported
version of consuming RNS as a 3rd party library.

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    rns = {
      url = "github:NotAShelf/rns";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

You may build `inputs.rns.packages.<system>.default` to access the dynamic
library and the header found in `include/`

### Zig

Zig "support" is still experimental, because I'm quite new to Zig. If you know
anything better, feel free to submit a pull request :)

```zig
// build.zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const lib = b.addSharedLibrary(.{
        .name = "config",
        .root_source_file = b.path("config.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Add include path for rns.h
    lib.addIncludePath(b.path("."));

    // Link against local librns.so shared library
    const lib_path = b.path("./librns.so");
    lib.addObjectFile(lib_path);

    // Add C library dependency
    lib.linkLibC();

    // Install the artifact
    b.installArtifact(lib);
}
```

```zig
// config.zig
const std = @import("std");
const c = @cImport({
    @cDefine("_GNU_SOURCE", "1");
    @cInclude("include/rns.h");
});

export fn configure() c_int {
    // Set basic options
    _ = c.nvim_set_global("mapleader", " ");
    _ = c.nvim_set_option_bool("number", 1);
    _ = c.nvim_set_option_bool("relativenumber", 1);
    _ = c.nvim_set_option_bool("expandtab", 1);

    // Set colorscheme
    _ = c.nvim_exec_command("colorscheme habamax");

    // Register plugins
    _ = c.register_plugin("telescope", "https://github.com/nvim-telescope/telescope.nvim");
    _ = c.register_plugin("plenary", "https://github.com/nvim-lua/plenary.nvim");

    // Install plugins
    _ = c.install_plugins();

    // Configure telescope
    _ = c.plugin_config_begin("telescope");
    _ = c.plugin_config_set_mapping("telescope", "i", "<C-j>", "move_selection_next");
    _ = c.plugin_config_add_keymap("n", "<leader>ff", "telescope", "find_files");
    _ = c.plugin_config_end();

    return 0;
}

// Lua module entry point
export fn luaopen_libconfig(L: ?*anyopaque) c_int {
    // Actually use the parameter to avoid compiler warning
    if (L != null) {}

    _ = configure();
    return 1;
}
```

```lua
-- init.lua

-- Load the RNS library
local ffi = require("ffi")

-- Load the configuration library
local config = package.loadlib("./zig-out/lib/libconfig.so", "luaopen_libconfig")
if not config then
  error("Failed to load libconfig.so")
end
config() -- Initialize the config

-- Or alternatively, load it directly without handling the configuration via Lua
-- local config = require('libconfig')

-- You can also call individual RNS functions if needed
-- Example:
-- ffi.cdef[[
--   int nvim_set_option_bool(const char *name, int value);
--   int register_plugin(const char *name, const char *url);
-- ]]
-- local rns = ffi.load('./librns.so')
-- rns.nvim_set_option_bool("number", 1)
-- rns.register_plugin("telescope", "https://github.com/nvim-telescope/telescope.nvim")
```

## Why?

Teehee.
