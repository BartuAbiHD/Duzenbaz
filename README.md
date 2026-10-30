# Duzen-Baz

A lightweight desktop app that brings your files, your clipboard and your eye health together under one roof. Everything stays on your computer: no internet connection is needed and there is no telemetry.

**Languages:** English · [Turkish](README-TR.md)

> Runs on **Windows, macOS and Linux**. Windows is the platform used day to day; macOS and Linux are built and tested automatically on every change, but have not been tried by hand yet. A few features depend on the desktop session, especially on Linux with Wayland (see [Platform status](#platform-status)).

It comes as two programs that share the same core and the same local data:

- **Duzen-Baz** — the desktop app (installers for Windows; on macOS and Linux you build it from source, see [Development](#development))
- **`duzenbaz-cli`** — a command line tool that is also an **MCP server**, so AI assistants such as Claude can use the same features

## Modules

### File Organizer
- Sorts the files of a folder (default: Downloads) into subfolders by type (images, documents, videos, music, archives, programs) and, if you like, by month.
- Shows a **preview** before moving anything; you can leave files out of the list.
- Every operation can be undone with one click from the **History** tab. No file is ever overwritten; name clashes become `name (1).ext`.
- Finds **duplicate files** by content ([BLAKE3](https://github.com/BLAKE3-team/BLAKE3) hash) and, with your confirmation, sends the extra copies to the **Recycle Bin**. Nothing is deleted permanently.

### Batch Renamer
- Drag and drop files or folders.
- Rules: date (including the photo's **EXIF date taken**), sequence number, prefix/suffix, find and replace, letter case.
- **Live preview** (old name → new name). Conflicting or invalid names are flagged before anything is applied.
- A rename is all-or-nothing, and the last one can be undone with one click.

### Clipboard History Manager
- Listens to the clipboard in the background and saves text, links and images.
- A searchable popup opens with **Ctrl+Shift+V** (configurable). The item you pick is pasted automatically.
- Pin items, give them a category and tags.
- Sensitive content: copies that password managers mark as "do not keep in history" are skipped, and anything copied from programs you choose (default: KeePass, Bitwarden, 1Password, …) is never saved. You can pause recording at any time.
- Data is kept only on your computer, **encrypted** (see [Security and privacy](#security-and-privacy)).

### Breaks and Eye Care
- Configurable break reminder (default: 20 seconds every 20 minutes). It only counts while you are at the computer; being away for long enough counts as a break.
- **Night mode:** warms the screen colour in the evening with a 30-minute fade in and out (Windows only for now).
- **Screen time report:** today's total, an hour-by-hour chart and a weekly chart. Only durations are stored, never which programs you use.

### General
- Lives in the system tray; closing the window sends the app to the tray. Optionally starts with Windows.
- Light / dark / system theme, Turkish and English interface (with a multi-language infrastructure), 12/24-hour clock option.
- Clear error messages.

## Command line tool and MCP server

`duzenbaz-cli.exe` uses the same code and the same local database as the desktop app, so operations you start in one can be undone in the other. It works without the desktop app running.

```bash
duzenbaz-cli organize plan   "C:\Users\me\Downloads"            # preview, changes nothing
duzenbaz-cli organize apply  "C:\Users\me\Downloads" --by-date  # asks before moving
duzenbaz-cli organize undo   <batch-id>
duzenbaz-cli organize history

duzenbaz-cli duplicates find  "C:\Users\me\Pictures" --recursive
duzenbaz-cli duplicates trash "C:\Users\me\Pictures" --recursive --keep oldest   # to the Recycle Bin

duzenbaz-cli rename preview "C:\Photos" --date exif --date-format %Y-%m-%d --number
duzenbaz-cli rename apply   "C:\Photos" --prefix "Holiday_" --case title --yes
duzenbaz-cli rename undo    <batch-id>

duzenbaz-cli clipboard list -q invoice -n 10
duzenbaz-cli clipboard get 42 > note.txt
duzenbaz-cli screen-time
```

- Commands that change something ask for confirmation, or need `--yes` when there is no terminal.
- Add `--json` to any command for machine-readable output. `--data-dir <DIR>` (or the `DUZENBAZ_DATA_DIR` environment variable) uses another data folder.
- Run `duzenbaz-cli --help` or `duzenbaz-cli <command> --help` for every option.

### Using it as an MCP server

`duzenbaz-cli mcp` speaks the [Model Context Protocol](https://modelcontextprotocol.io) over stdio (protocol versions 2025-06-18, 2025-03-26 and 2024-11-05).

**Claude Code**

```bash
claude mcp add duzenbaz -- "C:\path\to\duzenbaz-cli.exe" mcp --allow-dir "C:\Users\me\Downloads"
```

**Claude Desktop** (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "duzenbaz": {
      "command": "C:\\path\\to\\duzenbaz-cli.exe",
      "args": ["mcp", "--allow-dir", "C:\\Users\\me\\Downloads"]
    }
  }
}
```

Tools it offers:

| Tool | What it does | Changes files? |
|---|---|---|
| `organize_preview` | Shows what would move into type folders | no |
| `organize_apply` | Moves the files, returns a `batchId` | yes (undoable) |
| `organize_undo`, `organize_history` | Undo / list operations | undo changes files |
| `duplicates_find` | Finds groups of identical files | no |
| `duplicates_trash` | Keeps one file per group, others to the Recycle Bin | yes (Recycle Bin) |
| `rename_preview` | Shows old → new names, flags conflicts | no |
| `rename_apply` | Renames all files or none, returns a `batchId` | yes (undoable) |
| `rename_undo`, `rename_history` | Undo / list renames | undo changes files |
| `screen_time_report` | Screen time and break statistics | no |
| `clipboard_search` | Searches the clipboard history (**off by default**) | no |

Safety options:

- **Folders:** tools only work inside allowed folders. By default that is your Downloads folder; use `--allow-dir <DIR>` (repeatable) to allow others. `--allow-any-path` removes the restriction — use it with care, because the connected AI can then touch any file you can.
- **`--read-only`:** only the tools that never change anything are offered.
- **`--allow-clipboard`:** clipboard history may contain passwords or private text, so its tool is hidden unless you ask for it.
- Nothing is ever deleted permanently, every change can be undone, and the tool descriptions tell the assistant to preview first and confirm with you before applying.

## Technology

| Layer | Choice |
|---|---|
| Core | Rust, split into a shared library (`duzenbaz-core`), the desktop app and the CLI |
| Desktop shell | [Tauri 2](https://tauri.app) |
| Interface | Svelte 5 + Tailwind CSS 4 + Vite |
| Local data | SQLite (`rusqlite`) |
| Icons | [Lucide](https://lucide.dev) (kept as SVG files in the project, so they work offline) |

**Why these choices?** Tauri gives a much smaller installer (about 3 MB) and lower memory use than Electron, which matters for an app that runs in the tray all day. Svelte produces less code and a smaller bundle than React and keeps things like theme and language simple.

Main Rust crates: `walkdir`, `blake3`, `trash`, `kamadak-exif`, `arboard`, `clipboard-master`, `rusqlite`, `chacha20poly1305`, `keyring`, `clap`, `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`.

## Installation (for users)

Download from the [Releases](https://github.com/BartuAbiHD/Duzenbaz/releases) page:

- `Duzen-Baz_<version>_x64-setup.exe` — installs per user, no administrator rights needed
- `Duzen-Baz_<version>_x64_tr-TR.msi` — MSI package
- `Duzen-Baz_<version>_macos_x64.zip` — macOS app (Intel build; runs on Apple Silicon through Rosetta). Unzip and drag `Duzen-Baz.app` to Applications
- `Duzen-Baz_<version>_amd64.deb` — Linux package (Debian/Ubuntu, x86_64): `sudo apt install ./Duzen-Baz_<version>_amd64.deb`
- `duzenbaz-cli_<version>_linux_x64.tar.gz` — the command line tool and MCP server for Linux (x86_64)
- `duzenbaz-cli_<version>_macos_x64.tar.gz` — the command line tool and MCP server for macOS (Intel build)
- `duzenbaz-cli.exe` — the command line tool and MCP server; a single portable file, no installation needed

> The packages are not code-signed yet, so Windows SmartScreen may show an "unknown publisher" warning. Check the download against `SHA256SUMS.txt`.

The macOS app is not signed or notarized: on first launch right-click it and choose **Open**. The Linux packages are x86_64 only; on ARM (for example Raspberry Pi) build from source ([Development](#development)).

## Development

Requirements for every system: [Rust](https://rustup.rs) (stable) and Node.js 20+. In addition:

| System | What else you need |
|---|---|
| **Windows** | Visual Studio Build Tools (C++ workload) with the MSVC Rust toolchain; WebView2 (included in Windows 11) |
| **macOS** | Xcode Command Line Tools: `xcode-select --install` (macOS 11 or newer) |
| **Linux** (Debian/Ubuntu) | `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev patchelf` |

For other Linux distributions use the matching packages from the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#linux) list. The Debian/Ubuntu line above is exactly what the project's CI installs.

```bash
npm install
npm run tauri dev      # start the desktop app in development mode
```

Tests and checks:

```bash
cd src-tauri && cargo test --workspace     # Rust unit tests (core, app, CLI/MCP)
npm run check                              # Svelte / TypeScript check
```

Build the packages (output goes to `src-tauri/target/release/bundle`):

```bash
npm run tauri build                   # everything this system supports
npm run tauri build -- --bundles deb  # or just one: deb, rpm, appimage (Linux); app, dmg (macOS); nsis, msi (Windows)
```

| System | Packages |
|---|---|
| Windows | `.exe` (NSIS) and `.msi` |
| macOS | `.app` and `.dmg` (CI builds the `.app`) |
| Linux | `.deb`, `.rpm` and `.AppImage` (CI builds the `.deb`) |

Build the command line tool (output: `src-tauri/target/release/duzenbaz-cli`, with `.exe` on Windows):

```bash
cd src-tauri && cargo build --release -p duzenbaz-cli
```

On the first Windows build the bundler downloads the NSIS and WiX tools, and on Linux the AppImage tools (internet is only needed at build time).

#### Notes for macOS

- The app is not signed or notarized, so Gatekeeper blocks it on first launch. Right-click the app and choose **Open**, or run `xattr -dr com.apple.quarantine /path/to/Duzen-Baz.app` once.
- **Auto-paste** from the clipboard popup sends Cmd+V to the app in front, which macOS only allows after you grant the app **Accessibility** access (System Settings > Privacy & Security). Without it the item is still copied, you just paste it yourself.
- The Tauri bundler prints a warning that the identifier `com.duzenbaz.app` ends in `.app`. It is harmless; the identifier is kept so data folders stay the same on every system.

#### Notes for Linux

- Clipboard history needs a **Secret Service** provider (GNOME Keyring or KWallet) to keep its encryption key. Without one the clipboard history stays off instead of storing data it could not decrypt later.
- The system tray icon needs AppIndicator support. KDE and most desktops have it; GNOME needs the "AppIndicator and KStatusNotifierItem Support" extension.
- The default popup shortcut is **Ctrl+Alt+V** here (Ctrl+Shift+V is "paste" in terminals). You can change it in the Clipboard page.
- Night mode, idle detection, program detection and auto-paste use X11. They are not available in Wayland sessions, which do not let applications do these things (see the table below).

Browser shortcuts (F12, Ctrl+Shift+I, the context menu) are switched off on purpose. To debug, start the app with the `DUZENBAZ_DEVTOOLS=1` environment variable, which skips the Windows-side lock.

## Project structure

```
src/                        Svelte interface
  modules/                  organizer, renamer, clipboard, breaks, settings
  lib/                      shared components, stores, i18n (tr.json, en.json)
  assets/icons/             SVG icons (Lucide)
src-tauri/
  src/                      desktop app: Tauri commands, tray, popup windows, clipboard watcher
  crates/core/              shared core, no Tauri dependency
    src/organizer/          scanning, classification, duplicate detection, undo
    src/renamer/            rules, EXIF, safe renaming
    src/clipboard/          encrypted store, platform helpers
    src/breaks/             state machine, night mode, screen time
    src/db.rs, history.rs   SQLite connection with versioned migrations, shared undo log
  crates/cli/               duzenbaz-cli: command line and MCP server
```

To add a language, add a JSON file under `src/lib/i18n/` and register it in `index.ts`.

## Security and privacy

- The desktop app never connects to the internet; its content security policy keeps network access closed. There is no telemetry. The command line tool does not use the network either.
- **Clipboard data:** content and tags are encrypted with ChaCha20-Poly1305 before they are written to SQLite. The key is kept in the operating system's secure store (Windows Credential Manager, macOS Keychain, Linux Secret Service), not in the database file. Metadata such as item type, size and time is not encrypted.
- This is **not full-file encryption (SQLCipher)**. Field-level encryption was chosen because SQLCipher needs an OpenSSL build on Windows.
- Destination paths for file operations are recomputed in Rust rather than taken from the interface, and deletions only ever go to the Recycle Bin.
- If the app crashes, night mode can leave the screen tinted; just start the app again and turn night mode off.
- The MCP server restricts the folders it works in and never deletes permanently; see [Safety options](#using-it-as-an-mcp-server).

## Platform status

| Feature | Windows | macOS | Linux (X11) | Linux (Wayland) |
|---|---|---|---|---|
| File Organizer, Batch Renamer | yes | yes | yes | yes |
| Clipboard history (record, search, pin, tags) | yes | yes | yes | yes, if the compositor offers the data-control protocol |
| Ignore copies from chosen programs | yes | yes | yes | no |
| Skip password-manager copies automatically | yes | yes | no, use the ignore list | no |
| Auto-paste from the popup | yes | yes, needs Accessibility access | yes | no |
| Global popup shortcut | yes | yes | yes | mostly no (compositors block it) |
| Break reminder | yes | yes | yes | yes, but always counts as active (no idle detection) |
| Night mode | yes | yes | yes | no |
| Command line tool and MCP server | yes | yes | yes | yes |

How this was checked: on every change a GitHub Actions workflow runs on Windows, macOS and Linux. It builds the app, runs all tests, builds the command line tool and exercises it (including an MCP handshake), packages the app (`.deb` on Linux, `.app` on macOS) and starts it to check that it keeps running. On Linux it also runs the X11 code against a virtual X server (idle time, the night-mode colour change read back from the server, injected key presses) and the encryption key against a real Secret Service (GNOME Keyring). On macOS it runs the CoreGraphics and Objective-C code (idle time, front app, night mode).

What has only been tried by hand on Windows, because it needs a real desktop session: the popup window and its global shortcut, the tray icon, auto-paste into another application, the password-manager flags, and live clipboard monitoring. Please report anything that behaves differently on your system.

Some messages produced by the shared core (for example file errors) are still in Turkish, also in the command line tool.

## License

[MIT](LICENSE) © 2026 BartuAbiHD

Icons come from the [Lucide](https://lucide.dev) project and are distributed under the ISC license; the license text is in `src/assets/icons/LICENSE-lucide.txt`.
