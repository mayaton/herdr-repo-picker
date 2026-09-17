# herdr-repo-picker

A herdr plugin that opens a centered popup pane to fuzzy-search ghq repositories and jump to the matching workspace.

## Dependencies

- herdr (>= 0.7.4, for `placement = "popup"` support)
- ghq
- Rust 1.96+ (only needed to build from source)
- Claude Code CLI (used as the default `launch_command`)
- direnv (optional)

## Install

```sh
herdr plugin install mayaton/herdr-repo-picker
```

## Bind a key

Bind the `herdr-repo-picker:open-picker` action to a key in `~/.config/herdr/config.toml`. See the output of `herdr --default-config` for the format.

## Configuration

Run `herdr plugin config-dir herdr-repo-picker` to find the config directory, then place a `config.toml` there. Use `config.example.toml` as a template.

## Behavior

1. The bound key opens a centered popup pane.
2. Fuzzy-search and select a ghq repository, with the keyboard or the mouse.
3. On Enter (or double-click):
   - If a workspace with the same name already exists, it is focused.
   - Otherwise a new workspace is created and `claude` is launched in its initial pane (override via `launch_command`).
4. If the target repository has an unallowed `.envrc`, the launch is blocked.

### Mouse support

- Left-click a repository row to move the selection to it (like arrow keys).
- Double-click a row within 500ms to pick it (like Enter).
- Scroll the wheel to move the selection up/down one row at a time.
- Clicking outside the popup is not handled: it depends on whether the host
  herdr popup pane forwards outside clicks as terminal mouse events, which is
  unverified.

## Manual E2E checklist

- [ ] The popup opens via the bound key/prefix
- [ ] Typing ~3 characters narrows the list via fuzzy search
- [ ] Enter creates a new workspace and launches `claude` in its initial pane
- [ ] Picking the same repository again focuses the existing workspace
- [ ] Picking a repository with an unallowed `.envrc` shows the direnv guard screen and blocks the launch
- [ ] Esc / Ctrl-c closes the popup
- [ ] Left-click a row moves the selection without picking it
- [ ] Double-click a row picks it, same as Enter
- [ ] Scrolling the mouse wheel moves the selection and scrolls the list when it overflows the visible area

## Development

```sh
cargo test
cargo build --release
herdr plugin link .
```

## License

MIT
