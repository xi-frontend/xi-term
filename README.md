# xi-tui

`xi-tui` is a TUI frontend for [xi](https://github.com/google/xi-editor/).

## Current goals

- [X] basic editing
- [X] quit
- [X] scrolling (still some issues with "Page Up")
- [X] cursor handling
- [ ] selection
- [ ] yank/paste
- [ ] mouse support
- [ ] operating on files
    - [ ] opening a new file
    - [X] loading an existing file
    - [X] saving current file

`xi` is still very incomplete so it's hard to make long term plans.

## Screenshot

![screencast showing basic edition of this README.md file]("https://github.com/little-dude/xi-tui/blob/master/img/demo.gif")

## Credits

- @potocpav for the json-rpc client I stole from [xi_glium](https://github.com/potocpav/xi_glium).
- @ticki for [termion](https://github.com/ticki/termion), on which this project is based.
