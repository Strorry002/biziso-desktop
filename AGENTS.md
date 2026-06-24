# AGENTS.md -- Biziso Desktop (Windows)

Conventions for anyone (human or agent) working in this repository.

## What this is

A Windows desktop client for the Biziso platform, built as a Tauri 2 shell that
loads the Biziso web app. The home screen is the real platform dashboard
(biziso.com); the Touch messenger is the in-platform mod-touch web client. The
native layer provides single-instance, persisted window state and session, and
(in later phases) a system tray with unread badge, native notifications,
`biziso://` deep links and auto-update. This is a sibling to the Mac native app
(Strorry002/bizisointouch), but platform-first rather than a generic messenger.

## Phases

- F0 -- Scaffold: Tauri 2 app, loads biziso.com via WebView2, single-instance,
  persistent session, basic window chrome.
- F1 -- Native shell: tray + minimize-to-tray, native notifications bridged from
  the web, deep links (biziso://), auto-update, window-state persistence.
- F2 -- Platform home: open to the user dashboard, in-shell login flow.
- F3 -- Messenger integration: mod-touch in the shell, native toasts + tray
  unread badge.
- F4 -- Packaging: MSI/NSIS installer, Windows code-signing, auto-update feed.
- F5 -- Deeper native: background message receipt, richer OS integration.

## Rules

- No emoji in repo files, commits, PRs or docs.
- Hand-test each phase before starting the next; a type-check or compile alone
  is not enough.
- One PR ships one user-describable thing. Build locally before pushing.
- Keep the desktop client in sync with the mod-touch web messenger on the
  platform; reuse it rather than re-implementing.

## Layout

- `src-tauri/` -- the Rust/Tauri application.
  - `tauri.conf.json` -- window and bundle config; the main window URL is here.
  - `src/lib.rs` -- plugin registration and native wiring.
  - `capabilities/` -- Tauri permission capabilities.
- `src/` -- a minimal local fallback page; the window normally loads biziso.com.

## Develop

```
npm install
npm run tauri dev
```
