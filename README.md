# pdesk

Virtual desktops **per monitor** for Windows 11.

Windows already has virtual desktops, but switching one switches *every* screen at once.
pdesk gives each screen its own set of workspaces instead: you switch the screen under your
mouse, and the other screens stay exactly where they are. Like on macOS or GNOME.

Windows stay floating, just as they normally are. pdesk never tiles or rearranges them; it only
hides and shows them.

**There's a nicer version of this guide, with pictures: <https://canaangm.github.io/pdesk/>**

You need Git to download the code: `git clone https://github.com/CanaanGM/pdesk`.

## What you need

- Windows 11
- [Rust](https://rustup.rs) (pdesk is built from source on your machine)
- Admin rights once, for the install prompt

## Install

Open a **normal** terminal (not "Run as administrator") in this folder and run:

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1
```

It builds pdesk, asks once for admin (the UAC prompt), copies it to `C:\Program Files\pdesk`,
and sets it to start by itself every time you sign in. A small pdesk icon shows up in the tray.

To update after pulling new code, just run the same command again. Your workspaces are kept.

> Why admin? When an admin window (like Task Manager) has focus, Windows stops normal programs
> from hearing the keyboard. Running pdesk as admin keeps the shortcuts working everywhere.

## Shortcuts

Everything works on **the screen under your mouse**.

| Keys | What it does |
|---|---|
| `Win+Ctrl+Left` / `Right` | Previous / next workspace |
| `Win+1` … `Win+9` | Go straight to workspace N |
| `Win+Ctrl+Shift+Left` / `Right` | Send the focused window to the previous / next workspace, and go with it |
| `Win+Ctrl+Alt+Left` / `Right` | Send it there, but stay where you are |
| `Win+Shift+1` … `9` | Send the focused window to workspace N, and go with it |
| `Win+Ctrl+1` … `9` | Send it to workspace N, but stay |
| `Win+Shift+Left` / `Right` | Move a window to another screen (this one is Windows' own) |
| `Win+F` | **Finder**: search every window on every screen and workspace |
| `Win+Tab` | **Overview**: see every workspace of every screen at once |

Each switch shows a small row of dots for a moment, so you can see which workspace you're on.

Note: pdesk takes over `Win+Tab` (Windows' Task View) and `Win+1` … `9` (Windows' "open the
Nth taskbar app"). You can change any shortcut in the settings.

### Finder (`Win+F`)

Type part of a window's name or its app. The list starts with the windows you used most recently.

- `Enter`: go to that window (its screen switches to the right workspace)
- `Shift+Enter`: bring the window here instead
- `Ctrl+W`: close it
- `Esc`: close the finder

### Overview (`Win+Tab`)

Every screen shows a row of numbered tiles at the top, one per workspace, with live previews
of their windows. Below that, the current workspace's windows are spread out so none overlap.

- Click a window to go to it, or click a tile to switch that screen
- Drag a window onto a tile (on any screen) to move it there
- Arrow keys or `Tab` pick a window, `Enter` goes to it, a digit switches the workspace
- Hover a window and click its **X**, middle click it, or press `Ctrl+W` to close it
- `Esc`, a click on the background, or `Win+Tab` again closes the overview

The overview is the newest part and is still being worked on.

## Touchpad swipes

You can make the four finger swipe switch pdesk's workspaces:
**Settings → Bluetooth & devices → Touchpad → Advanced gestures**, then set the four finger
swipe left to `Win+Ctrl+Right` and swipe right to `Win+Ctrl+Left` (or the other way round,
whichever feels natural).

## Settings

Right click the tray icon → **Open settings**. The file is
`%LOCALAPPDATA%\pdesk\settings.toml`, and every line has a comment saying what it does. Things
you can change:

- How many workspaces each screen has (2 to 9, default 4)
- Every shortcut, or turn one off
- The dots indicator: on or off, and how long it stays
- Whether the Copilot key acts as `Right Ctrl`
- The overview's window titles (`"hover"`, `"always"` or `"none"`) and whether it updates live

pdesk reads the file once at start, so after editing it run `install.ps1` again (or sign out
and back in). If you type a bad value, only that one setting goes back to its default; the log
says which.

## Good to know

- Windows on other workspaces disappear from the taskbar and from Alt+Tab, so each screen only
  shows what's on it. (Admin windows like Task Manager are the exception and still show in
  Alt+Tab.)
- If a hidden window pops up by itself (a chat message, say), pdesk switches its screen to that
  window's workspace.
- After a restart, pdesk puts reopened windows back on the workspaces they were on, for the
  first few minutes after you sign in.
- The tray icon menu has **Open settings**, **Open log** and **Quit**.

## Stopping and uninstalling

**Quit** from the tray (or `& "$env:ProgramFiles\pdesk\pdesk.exe" --quit`) stops pdesk and
shows every hidden window, so nothing is ever lost. It also forgets which window was on which
workspace, on purpose.

To remove it completely, quit it first, then in an **admin** terminal:

```powershell
Unregister-ScheduledTask -TaskName pdesk -Confirm:$false
Remove-Item -Recurse "$env:ProgramFiles\pdesk"
```

Your settings and log stay in `%LOCALAPPDATA%\pdesk`; delete that folder too if you want.

## If something goes wrong

- **A window seems lost**: press `Win+F` and search for it, or open the overview.
- **Shortcuts stopped working**: check the tray icon is there. If not, run `install.ps1` again.
- **Everything is weird**: Quit from the tray. Every window comes back. Then run `install.ps1`.
- The log (tray → **Open log**) says what pdesk did and why.

pdesk uses an undocumented part of Windows to hide windows. It works on Windows 11 25H2, but a
big Windows update could change that.
