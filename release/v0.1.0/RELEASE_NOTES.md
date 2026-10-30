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
| `duzenbaz-cli.exe` | Command line tool and MCP server, no installation needed |
| `SHA256SUMS.txt` | Checksums of the files above |

Requirements: Windows 10/11 (64-bit).

### Known notes
- **The files are not code-signed.** Windows SmartScreen may show an "unknown publisher" warning. Check your download against `SHA256SUMS.txt`.
- Windows only for now. Night mode and auto-paste do not work on macOS or Linux yet.
- This is a pre-release. The desktop app and the command line tool were tried on a Windows 11 development machine; the installers were not tried on a clean machine. The files were built by this repository's GitHub Actions workflow on a clean Windows runner, after all tests passed.
- Clipboard data is encrypted field by field (not full-file encryption); details are in the [README](https://github.com/BartuAbiHD/Duzenbaz#security-and-privacy).
- Some messages from the shared core (for example file errors) are still in Turkish, also in the command line tool.

### Checksums (SHA-256)
```
e3bad363e834b7f888a71b3486bcdde78fb205acc8c7aa45fdbda54ab0cc0f51  Duzen-Baz_0.1.0_x64-setup.exe
dba70ae496a249e030a6ffb6da3da24a5086f53ac3f53d02060eb6ee800774d2  Duzen-Baz_0.1.0_x64_tr-TR.msi
eb420f1054ebe6f1791720858950b01eb84f16df9902160fa8e8ae26d24e45fb  duzenbaz-cli.exe
```
