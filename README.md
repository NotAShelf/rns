# RNS

Something something write your Neovim configurations in C. Highly experimental,
almost fully untested. If you look at this and go "wow what a great idea, I will
start using this project right this instance!" I have unfortunate news for your
friends and family and it involves you being sent to a mental hospital.

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
cp target/release/librns.so ./librns.so
```

Write your configuration in C, and then compile it with `librns` in your library
path. Note that you will also need `librns.so` in Neovim's `LD_LIBRARY_PATH`
later on.

```bash
# Replace target/release with where you have placed librns
gcc -o init.so -shared -fPIC init.c -Ltarget/release -lrns -Wl,-rpath,./
```

Now you can load your new, compiled configuration:

```lua
-- init.lua
package.cpath = package.cpath .. ";./?.so"
local init = require("init")
```

```bash
# Ensure that librns.so is in Neovim's library path before
# you run this. Otherwise your configuration will not be
# loaded.
nvim --clean -u path/to/your/init.lua
```

Now you may evaluate your life choices and consider how you even got here!
Enjoy.
