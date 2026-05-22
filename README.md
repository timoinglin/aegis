# Aegis

A friendly Windows desktop app for **EmuCoach MoP-repack server owners**. It turns
the common database and server chores — backups, restores, account management —
into buttons, so you don't need the command line, HeidiSQL, or any SQL knowledge.

Aegis talks to your server using the **MySQL tools that already came with your
repack**, and (for account tasks) your server's built-in Remote Access. It never
makes up its own database password format, and it always takes a safety backup
before anything destructive.

> Status: **v0.1 skeleton.** The Health system, Settings, and connection checks
> are in place. Backups, restore, account management and the updater are landing
> next — see the build plan.

## What's here so far

- **Health / Status spine** — one place that always tells you what needs your
  attention: database reachable, repack tools found, Remote Access reachable,
  worldserver running. Each issue explains *what's wrong, why it matters, and how
  to fix it*, with a Re-check button. No raw error messages, ever.
- **Settings** — database and Remote Access connection details, plus auto-detect
  for your repack's `_Server` folder. Stored in `%APPDATA%\Aegis`.

## Running it (development)

You need [Node.js](https://nodejs.org) and the
[Rust toolchain](https://rustup.rs) (Rust is required to build the Tauri backend).

```powershell
npm install
npm run app:dev      # launches the desktop app in dev mode
```

Build a release `.exe`:

```powershell
npm run app:build
```

The first build needs an app icon — see [src-tauri/icons/README.md](src-tauri/icons/README.md).

### "Windows protected your PC" / unknown publisher

Aegis is self-signed, so Windows SmartScreen may warn you the first time you run
it. That's expected for a small independent tool. Click **More info → Run anyway**.

## Support

Aegis is a solo project, so support is via the README and **GitHub Issues** rather
than DMs. Every operation is written to a log file in `%APPDATA%\Aegis\logs` —
attach the relevant log to an issue and it's much easier to help.

## Layout

```
src/          React + TypeScript frontend (UI, Health surface)
src-tauri/    Rust backend (commands = IPC surface, services = logic)
```
