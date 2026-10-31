## Duzen-Baz 0.1.0 — first release

**Languages:** English · [Turkish](RELEASE_NOTES-TR.md)

A lightweight desktop app that brings your files, your clipboard and your eye health together. Everything stays on your computer: no internet connection is needed and there is no telemetry. It also ships a command line tool that doubles as an **MCP server**, so AI assistants such as Claude can use the same features.

### Modules
- **File Organizer:** sorts a folder by type and date, finds duplicate files (sent to the Recycle Bin, never deleted permanently), with preview and undo
- **Batch Renamer:** drag and drop, date (including EXIF), sequence number, prefix/suffix, find and replace, live preview and undo
- **Clipboard History:** encrypted local history, a searchable popup on `Ctrl+Shift+V`, pinning, categories and tags, ignoring password-manager content
- **Breaks and Eye Care:** break reminder, night mode, daily screen time report

### New: `duzenbaz-cli` and MCP server
`duzenbaz-cli.exe` is a single portable file that uses the same core and the same local data as the desktop app.

- Commands: `organize`, `duplicates`, `rename`, `clipboard`, `screen-time` (add `--json` for machine-readable output)
- `duzenbaz-cli mcp` runs a [Model Context Protocol](https://modelcontextprotocol.io) server over stdio with 11 tools (clipboard search is a 12th, off unless you pass `--allow-clipboard`)
- Safe by default: tools only work in allowed folders (default: Downloads, change with `--allow-dir`), `--read-only` hides every tool that changes something, files are only ever sent to the Recycle Bin, and every change can be undone

```bash
claude mcp add duzenbaz -- "C:\path\to\duzenbaz-cli.exe" mcp --allow-dir "C:\Users\me\Downloads"
```

See the [README](https://github.com/BartuAbiHD/Duzenbaz#command-line-tool-and-mcp-server) for all commands and the Claude Desktop configuration.

### Downloads
| File | What it is |
|---|---|
| `Duzen-Baz_0.1.0_x64-setup.exe` | Recommended installer; installs per user, no administrator rights needed |
| `Duzen-Baz_0.1.0_x64_tr-TR.msi` | MSI package |
| `Duzen-Baz_0.1.0_macos_x64.zip` | macOS app (Intel build, runs on Apple Silicon through Rosetta); unzip and drag to Applications. Not signed: right-click and choose Open on first launch |
| `Duzen-Baz_0.1.0_amd64.deb` | Linux package (Debian/Ubuntu, x86_64; built on Xubuntu 22.04); install with `sudo apt install ./Duzen-Baz_0.1.0_amd64.deb` |
| `duzenbaz-cli_0.1.0_linux_x64.tar.gz` | Command line tool and MCP server for Linux (x86_64); extract it and run `./duzenbaz-cli` |
| `duzenbaz-cli_0.1.0_macos_x64.tar.gz` | Command line tool and MCP server for macOS (Intel build); extract it and run `./duzenbaz-cli` |
| `duzenbaz-cli.exe` | Command line tool and MCP server, no installation needed |
| `SHA256SUMS.txt` | Checksums of the files above |

Requirements: Windows 10/11 (64-bit), macOS 11 or later for the zip, or a 64-bit Debian/Ubuntu-based Linux for the `.deb`.

### Known notes
- **The files are not code-signed.** Windows SmartScreen may show an "unknown publisher" warning. Check your download against `SHA256SUMS.txt`.
- Windows and macOS packages are provided; Linux packages are for x86_64 (not tried on ARM; build from source there). The macOS app was built on a Mac and is not notarized. Night mode works on Linux only under X11.
- This is a pre-release. The desktop app and the command line tool were tried on a Windows 11 development machine; the installers were not tried on a clean machine. The Windows installers were built locally on the development machine from the tagged sources (the Linux and macOS packages were built on their own machines); none of the packages were built by CI.
- Clipboard data is encrypted field by field (not full-file encryption); details are in the [README](https://github.com/BartuAbiHD/Duzenbaz#security-and-privacy).
- Some messages from the shared core (for example file errors) are still in Turkish, also in the command line tool.

### Checksums (SHA-256)
```
783fe16284d3f8a2d5e9b435b06acf9b32f69c65fb0468a2534a7a97d165abba  Duzen-Baz_0.1.0_x64-setup.exe
c102adc7b8c5e8c061c883d8deec0dd19b46a165ec8a5ec25e5f0fe290ddc7dd  Duzen-Baz_0.1.0_x64_tr-TR.msi
760e6ffc4ac272959139566e00b7829496540854b45ca9c3ea704944f3138348  duzenbaz-cli.exe
efaaebff093d0549059f8786c13f2a5aae0e0ffd54d9648688b5adb37c2aa1b4  Duzen-Baz_0.1.0_macos_x64.zip
0c9a2118ef78d19094ac3513157d3508164e265f9734122512c3134023508af5  duzenbaz-cli_0.1.0_macos_x64.tar.gz
7abf07640f74c33d615844976106f849788cc97b19a7ceb6218d176a98f36753  Duzen-Baz_0.1.0_amd64.deb
b4098ab2336aee3c7c1bc71a4a0999356258e3fbd5052a9b6f697b70882ebce0  duzenbaz-cli_0.1.0_linux_x64.tar.gz
```
