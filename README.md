# RNS

Something something write your Neovim configurations in C something. _Highly_
experimental, almost fully untested. If you look at this and go "wow what a
great idea, I will start using this project right this instance!" I have
unfortunate news for your friends and family and it involves you being sent to a
mental hospital.

Inspired by [CatNvim](https://github.com/rewhile/CatNvim).

## Usage

> [!WARNING]
> The API might change, at any given time and without notice. This is not
> intended for public use (or any use, really) and I will not make any attempts
> to keep the library or its API stable.

Build rns as a shared library:

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

## Example Configuration

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

## Why?

Teehee.
