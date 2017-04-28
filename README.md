# WARNING

I have not worked on this for a while, it's not working with recent Xi versions.

The latest SHA on [xi-core](https://github.com/google/xi-editor/) that we are
compatible with is 429da468f6819765e969807a41ff19680316ddce. Follow the
installation instructions below (in "Running the Core"), but do a `git checkout
429da468f6819765e969807a41ff19680316ddce` first. If you have already done a
`cargo install` for some other version of the core, you may need to do `cargo
install --force` to replace the existing installation.

# xi-tui

[![Build Status](https://travis-ci.org/little-dude/xi-tui.svg?branch=master)](https://travis-ci.org/little-dude/xi-tui)

`xi-tui` is a TUI frontend for [xi](https://github.com/google/xi-editor/).

## Current goals

- [X] basic editing
- [X] quit
- [X] cursor handling
- [ ] scrolling
    - [X] with keyboard (still some issues with "Page Up")
    - [ ] with mouse
- [ ] selection
    - [X] with mouse
    - [ ] with keyboard
- [ ] yank/paste
- [ ] operating on files
    - [ ] opening a new file
    - [X] loading an existing file
    - [X] saving current file

## Screenshot

![screencast showing basic edition of this README.md file](https://github.com/little-dude/xi-tui/blob/master/img/demo.gif)

## Credits

- @potocpav for the json-rpc client I stole from [xi_glium](https://github.com/potocpav/xi_glium).
- @ticki for [termion](https://github.com/ticki/termion), on which this project is based.

## Running the Core

The frontend assumes that you have installed the [core editor](https://github.com/google/xi-editor)
and is available in your PATH. The following should suffice:

```bash
git clone https://github.com/google/xi-editor
cd xi-editor/rust
cargo install
# You now need to add ~/.cargo/bin to your PATH (this is where `cargo install`
# places binaries). In your .bashrc (or equivalent), add `export PATH=$PATH:~/.cargo/bin`
```

## Caveats

### Tabs

We assume tabs (`\t`) are 4 columns large. It that is not the case in your
terminal, the cursor position will be inaccurate. On linux, to set the `\t`
width to four spaces, do:

```
tabs -4
```

### Line wrapping

Line wrapping is completely unsupported for now. Lines that are too long will
mess up the output. I'm not even sure if line wrapping should occur in the
backend or in the frontend.
