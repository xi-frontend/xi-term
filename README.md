# xi-tui

[![Build Status](https://travis-ci.org/little-dude/xi-tui.svg?branch=master)](https://travis-ci.org/little-dude/xi-tui)

`xi-tui` is a terminal frontend for [xi](https://github.com/google/xi-editor/).

It is experimental and under development, so don't expect anything magical (yet!).

## Installation

The frontend assumes that you have installed the
[core editor](https://github.com/google/xi-editor)
and is available in your PATH. The following should suffice:

```bash
git clone https://github.com/google/xi-editor
cd xi-editor/rust
cargo install

# if you want syntax highlighting, you need to install the syntect plugin:
cd syntect-plugin
cargo install

# You need to add ~/.cargo/bin to your PATH
# (this is where `cargo install` places binaries).
# In your .bashrc (or equivalent), add `export PATH=$PATH:~/.cargo/bin`
```

Then you can clone this repository and run the frontend with
`cargo run --release -- <your_file>`.
`your_file` can be an existing file or any dummy name.

## Logging

For debugging, it can be useful to have logs.
You can specify a location for log files `xi-tui` with `-l <logfile>`.
Two files will be written:

- `<logfile>`: all the `xi-tui` logs
- `<logfile>.rpc`: the RPC messages exchanged between the core and the frontend

## Screenshots

![a python file](.github/python.png)

![the README file](.github/README.png)

## Shortcuts

For now, there are only two shortcuts:

- `^w` saves the current view
- `^c` exits

## Caveats

### Tabs

We assume tabs (`\t`) are 4 columns large. It that is not the case in your
terminal, the cursor position will be inaccurate. On linux, to set the `\t`
width to four spaces, do:

```
tabs -4
```

### Colors

If you have the `syntect` plugin installed, colors will be enabled by default, with two caveats:

- you must have true colors enabled. Otherwise, some portions of text won't be displayed
- the default theme is for dark backgrounds
