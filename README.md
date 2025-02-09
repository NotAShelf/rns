# NRS

Something something write your Neovim configurations in C. Highly experimental,
almost fully untested. If you look at this and go "wow what a great idea, I will
start using this project right this instance!" I have unfortunate news for your
friends and family and it involves you being sent to a mental hospital.

Inspired by [CatNvim](https://github.com/rewhile/CatNvim).

## Usage

Build rns as a shared library:

```bash
cargo build --release
```

Move it to your working directory, and you may begin interfacing with the
resulting shared library in C.

```bash
cp target/release/libinit.so ./init.so
```

Use it in your `init.c`, for example, as follows:

```c
// init.c
#include <lua.h>
#include <lauxlib.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

// External Rust functions
extern void run_cmd(const char *cmd);
extern char *concat_str(const char *s1, const char *s2);
extern bool os_isdir(const char *path);
extern const char *get_xdg_home(int mode);
extern void opt(const char *key, const char *old_val, const char *new_val);
extern void require_setup(const char *module, const char *config);

int luaopen_init(lua_State *L) {
    // Get the path for lazy.nvim installation
    char *lazypath = concat_str(get_xdg_home(1), "/lazy/lazy.nvim/");

    // Clone lazy.nvim if it's not installed
    if (!os_isdir(lazypath)) {
        char *cmd = concat_str(
            "git clone --filter=blob:none https://github.com/folke/lazy.nvim.git --branch=stable ",
            lazypath
        );
        system(cmd);
        free(cmd);
    }

    // Add lazy.nvim to runtimepath
    opt("runtimepath", "DEFAULT_RUNTIMEPATH", lazypath);

    // Load and configure LazyVim plugins
    require_setup("lazy",
        "{"
        "  spec = {"
        "    {"
        "      'catppuccin/nvim',"
        "      name = 'catppuccin',"
        "      opts = {"
        "        color_overrides = {"
        "          mocha = {"
        "            base = '#000000',"
        "            mantle = '#000000',"
        "            crust = '#000000'"
        "          }"
        "        }"
        "      }"
        "    },"
        "    {"
        "      'LazyVim/LazyVim',"
        "      import = 'lazyvim.plugins',"
        "      opts = { colorscheme = 'catppuccin' }"
        "    }"
        "  },"
        "  install = {"
        "    colorscheme = { 'catppuccin' }"
        "  }"
        "}"
    );

    return 1;
}
```

Compile it:

```bash
# Replace target/release with where you have placed librns
gcc -o init.so -shared -fPIC init.c -Ltarget/release -lnrs
```

Now you can load your new, compiled configuration:

```lua
-- init.lua
package.cpath = package.cpath .. ";/path/to/init.so"
require("init")
```

Enjoy!
