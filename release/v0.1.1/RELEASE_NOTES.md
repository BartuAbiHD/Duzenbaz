## Duzen-Baz 0.1.1

**Languages:** English · [Turkish](RELEASE_NOTES-TR.md)

### New
- **Update check (Settings → Updates).** Off by default. When you turn it on, the app asks GitHub for the latest version number when it starts and can download and install a newer version after you confirm. Every update is verified with a signature. While it is off, the app never connects to the internet. On Linux (`.deb`) the app only tells you about a new version and opens the download page.

### Fixed
- **Break reminder never appeared.** The 20-minute counter was reset by any pause of more than a minute without keyboard or mouse input, for example while reading. Now the counter only pauses; it resets after 5 minutes away from the computer.
- **Linux: window stayed blank or ignored clicks in virtual machines.** Software rendering is now the default for the web view (`WEBKIT_DISABLE_COMPOSITING_MODE=1`; set it to `0` to turn it off).
- **Linux: the clipboard key could freeze start-up** while the system keyring was asked for a password; it is now loaded in the background.
- **Linux: pasting from the clipboard history.** The copied item is kept available until the target program has received it, and focus is given back to the previous window before `Ctrl+V` is sent. If pasting still fails on your system, start the app with `DUZENBAZ_DEBUG=1` and open an issue with the `pano:` lines.
- The Windows installer now installs to `Duzen-Baz` instead of `Düzen-Baz`.

### Downloads
| File | What it is |
|---|---|
| `Duzen-Baz_0.1.1_x64-setup.exe` | Recommended installer; installs per user, no administrator rights needed |
| `Duzen-Baz_0.1.1_x64_tr-TR.msi` | MSI package |
| `duzenbaz-cli.exe` | Command line tool and MCP server, no installation needed |
| `latest.json` | Update information read by the app (not for download) |
| `SHA256SUMS.txt` | Checksums of the files above |

Requirements: Windows 10/11 (64-bit). macOS and Linux packages for this version are added separately.

### Known notes
- **The files are not code-signed** (the update signature is separate from Windows code signing). Windows SmartScreen may show an "unknown publisher" warning. Check your download against `SHA256SUMS.txt`.
- Version 0.1.0 has no update check; install 0.1.1 once by hand, later versions can update themselves.

### Checksums (SHA-256)
```
a129a9c61c39e928c2658643512528c4b758e88e1999535b87485a14b2b2d8aa  Duzen-Baz_0.1.1_x64-setup.exe
621763d51a217846695a10065f37e757ffd9b756477937681fd7d4c28cb90218  Duzen-Baz_0.1.1_x64_tr-TR.msi
39413522ac955ccd52c726630c93a9797600baed7960c478837f25df230fd228  duzenbaz-cli.exe
```
