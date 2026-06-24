# Biziso Desktop (Windows)

Windows desktop client for the Biziso platform. A Tauri 2 shell whose home is
the real Biziso web platform -- the user dashboard, modules and widgets from
biziso.com -- with the Touch messenger (mod-touch) integrated. A Biziso desktop
client, not a generic messenger.

## Architecture

- Tauri 2: a Rust core plus the OS WebView2 runtime (preinstalled on Windows 11).
- The main window loads `https://biziso.com`, so the home is the user's real
  dashboard with zero re-implementation.
- The native layer adds what a browser tab cannot: single-instance, persisted
  window state and session, and -- in later phases -- a system tray with an
  unread badge, native notifications, `biziso://` deep links and auto-update.

See `AGENTS.md` for conventions and the full phase plan.

## Prerequisites

- Windows 10/11 with the WebView2 runtime (preinstalled on Windows 11).
- Rust (stable `x86_64-pc-windows-msvc`) and the MSVC C++ build tools.
- Node.js 18+ and npm.

## Develop

```
npm install
npm run tauri dev
```

## Build

```
npm run tauri build
```

Produces the app and installers under `src-tauri/target/release/`.
