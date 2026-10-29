---
name: ferris-windows
description: Implements or debugs Windows paths, encoding, DLL loading, privileges, Win32/COM/PInvoke, and PowerShell 7 commands. Use when the target host or API is Windows. Not for a non-Windows target.
---

# Windows Engineering

PowerShell 7 (`pwsh`) is the default shell unless the target or repository requires Windows PowerShell 5.1 or cmd. Apply only the sections the task touches.

## Shell execution

- Stop dependent work when a native command fails. In PowerShell 7, `&&` checks `$LASTEXITCODE`, but a native nonzero status becomes an error record only when `$PSNativeCommandUseErrorActionPreference` is `$true`; the default is `$false`. Handle tool-specific success codes, such as robocopy, explicitly. Cmdlet error handling is separate.
- Use literal paths and direct native invocation with argument arrays where possible. `Start-Process -ArgumentList` joins array elements into one command-line string; preserve the target program's required quoting for spaces and embedded quotes. PowerShell is not a POSIX shell.
- A name that `Get-Command` resolves under WindowsApps may be an app-execution alias, such as the Store redirector for `python` or `python3`, rather than the installed tool. Invoke the real executable or launcher.

## Paths, open files, and DLLs

- Long-path opt-in (system policy plus application manifest) covers many Win32 file APIs, not every API, shell, or runtime. Supported extended paths are absolute `\\?\C:\...` or `\\?\UNC\server\share\...`, with backslashes and no `.`/`..`; the prefix disables normal normalization. Never blindly prepend it.
- Names are usually case-preserving/insensitive, but NTFS directories can be case-sensitive. No case-only source collisions or lowercasing as universal identity; use a two-step Git move for case-only renames. Avoid device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, including extensions), trailing spaces/dots, and invalid filename characters.
- Open-file deletion/replacement depends on the operation and existing share modes, notably `FILE_SHARE_DELETE`. Identify the denying handle/process, including LNK1168 holders; retries help only if it will release the handle.
- Load non-system DLLs from trusted absolute paths with appropriate `LoadLibraryExW` `LOAD_LIBRARY_SEARCH_*` flags for dependencies. The top-level path does not secure transitive loads. Avoid current-directory/PATH searching and process-wide search-path mutation around individual calls.
- Stay unelevated except for the one operation that needs it; elevation and persistent machine-wide changes need approval. Symlinks may need Developer Mode or privilege; junctions cover only directories, and copies lose update semantics. A fallback that preserves behavior needs no extra approval; one that changes link or update semantics does, and otherwise report the limitation.

## Encoding and terminal I/O

Source literals, internal strings, console text, and external bytes are separate contracts. Specify encoding at both ends of text files/pipes, not from the user's locale:

| Boundary | Easily missed distinction |
|----------|---------------------------|
| MSVC literals | `/utf-8` sets source and execution encoding; a BOM identifies source only |
| Win32 text | Prefer `W` APIs; distinguish byte counts, UTF-16 code units, and terminators. Explicit `CP_UTF8` conversions must honor error/length contracts; no `CP_ACP` for durable/protocol text |
| Console versus redirection | Unicode runtime support or `ReadConsoleW`/`WriteConsoleW` handles console text; UTF-8 console bytes need the appropriate code page (`SetConsoleOutputCP(CP_UTF8)` for output). Redirected handles are byte streams, not consoles; code-page changes do not transcode files/pipes |
| Legacy narrow APIs | An `activeCodePage` UTF-8 manifest depends on supported Windows/API behavior; GDI is an exception |
| PowerShell text/native | `[Console]::OutputEncoding` controls native-output decoding where it occurs; `$OutputEncoding` controls text sent to native stdin |
| PowerShell 7 text files | Default to UTF-8 without BOM; choose the writer/encoding required by the consumer |

PowerShell 7.4+ preserves bytes for native stdout redirected to a file or piped directly to another native command. Do not replace that path with `Out-File`/text processing for binary data. Merging stderr with `2>&1` makes the combined stream text again.

For CLI styling, detect a terminal and enable `ENABLE_VIRTUAL_TERMINAL_PROCESSING` where required; keep escape sequences out of redirected/machine-readable output.

### Legacy shell targets only

For Windows PowerShell 5.1 compatibility, replace `&&`/`||` with separate commands and explicit checks. Cmdlet encoding defaults vary; `-Encoding utf8` emits a BOM, generally needed for non-ASCII 5.1 scripts. For non-ASCII cmd scripts, UTF-8 plus `chcp 65001` on an ASCII line must take effect before non-ASCII commands are parsed. In batch, check `if errorlevel 1` for tools where nonzero means failure.

## API and GUI contracts

Use `FAILED`/`SUCCEEDED` for `HRESULT`, `ERROR_SUCCESS` for `LSTATUS`, and the documented sentinel for other results. Capture `GetLastError()` immediately only where promised. Match owned handles/buffers/COM allocations to their release functions; never release borrowed/pseudo-handles. Initialize size/version fields and keep pointer-sized values untruncated.

Declare GUI DPI awareness in the manifest, using `PerMonitorV2` where supported, and use per-window DPI. Respect framework setup instead of racing it with late runtime configuration.

References: [long paths](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation), [case sensitivity](https://learn.microsoft.com/en-us/windows/wsl/case-sensitivity), [DLL search](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order), [PowerShell byte redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection), [Start-Process arguments](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/start-process#-argumentlist).
