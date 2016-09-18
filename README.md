# xi-tui

`xi-tui` is a TUI frontend for [xi](https://github.com/google/xi-editor/).

## Current goals

- [X] basic editing
- [X] quit
- [X] cursor handling
- [-] scrolling
    - [X] with keyboard (still some issues with "Page Up")
    - [-] with mouse
- [-] selection
    - [X] with mouse
    - [ ] with keyboard
- [ ] yank/paste
- [-] operating on files
    - [ ] opening a new file
    - [X] loading an existing file
    - [X] saving current file

## Screenshot

![screencast showing basic edition of this README.md file](https://github.com/little-dude/xi-tui/blob/master/img/demo.gif)

## Credits

- @potocpav for the json-rpc client I stole from [xi_glium](https://github.com/potocpav/xi_glium).
- @ticki for [termion](https://github.com/ticki/termion), on which this project is based.

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
