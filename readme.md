<p align="center">
  <img src="icons/app/windowlasso.svg" alt="WindowLasso" width="128" height="128">
</p>

<h1 align="center">WindowLasso</h1>

<p align="center">
  <strong>Recover and manage windows that have wandered off-screen</strong>
</p>

---

A simple Windows utility to find and bring back windows that have ended up on disconnected monitors or are otherwise lost off-screen.

## Features

- Automatically detects off-screen and minimized windows
- Move windows to any connected monitor with a click
- Global hotkeys for quick access
- System tray integration
- Multi-language support (English, Spanish, French, German, Japanese, Chinese)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + Alt + L` | Open lasso window picker |
| `Ctrl + Alt + R` | Refresh window list |
| `Ctrl + Alt + P` | Move first off-screen window to primary monitor |
| `Ctrl + Alt + A` | Move all off-screen windows to primary monitor |
| `Ctrl + Alt + C` | Center current window on its monitor |
| `Ctrl + Alt + N` | Move current window to next monitor |

All hotkeys can be customized or disabled in Settings.

## Building

```bash
cargo build --release
```

## License

MIT