<p align="center">
  <img src="assets/img/logo.png" width="130" alt="Aegis logo">
</p>

<h1 align="center">Aegis</h1>

<p align="center">
  A friendly Windows app that makes running your <strong>EmuCoach Mists of Pandaria</strong> server easy —
  back up your database, create accounts, set GM levels, and more, all with simple buttons.
  <br>No command line. No HeidiSQL. No SQL knowledge needed.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-Windows-0078D6?logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/built%20with-Tauri-24C8DB?logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/for-EmuCoach%20MoP%205.4.8-2dd4bf" alt="EmuCoach MoP 5.4.8">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License: MIT">
  <img src="https://img.shields.io/badge/status-In%20Development-orange" alt="Status: In Development">
  <img src="https://img.shields.io/github/v/release/timoinglin/aegis?label=release&color=8B4513" alt="GitHub Release">
</p>

> 💛 **Find Aegis useful?** It's free and open-source, built in spare time for the EmuCoach community. If it saved you a headache, a coffee genuinely helps it keep growing — see [Support the Project](#support-the-project).
>
> [![Support the project on Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/kneuma)

> ⚠️ **In development** — Aegis is being built in the open. The first downloadable release is coming soon. Until then you can [build it from source](#building-from-source) or watch the repo for the release.

## Table of Contents

- [What is Aegis?](#what-is-aegis)
- [What it can do](#what-it-can-do)
- [Preview](#preview)
- [Download & Install](#download--install)
- [First time you open it](#first-time-you-open-it)
- [Keeping it up to date](#keeping-it-up-to-date)
- [Is it safe?](#is-it-safe)
- [Troubleshooting & Help](#troubleshooting--help)
- [Building from source](#building-from-source)
- [Support the Project](#support-the-project)
- [License](#license)

## What is Aegis?

If you run your own EmuCoach MoP 5.4.8 repack, you already have full control over it — but doing
the everyday jobs usually means typing database commands, installing extra programs, or following
fiddly guides.

**Aegis turns those jobs into buttons.** It talks to your server using the tools that already came
with your repack, so there's nothing extra to install and nothing to configure by hand. It's made
for server owners of *every* skill level — if you can install a repack, you can use Aegis.

## What it can do

- 🩺 **Health dashboard** — One screen that tells you, at a glance, whether everything's working:
  is your database reachable, are the server tools where Aegis expects them, is Remote Access on.
  If something's wrong, Aegis explains it in plain English and tells you exactly how to fix it —
  **never a scary error message**.
- 💾 **One-click backups** — Save a complete copy of your accounts, characters and world to a single
  file. Aegis shows you the size and a quick sanity check so you know the backup actually worked.
- ♻️ **Safe restore** — Put a backup back if something goes wrong. Aegis always takes a fresh safety
  backup *first*, asks you to confirm, and won't let you restore while your server is running (which
  could corrupt your data). Belt and braces.
- 👥 **Account management** — Create accounts, set someone's GM level, or reset a password — without
  touching the database. Passwords are handled by your server itself, so logins always work.
- ⚙️ **Settings & Test Connection** — Aegis tries to find your repack automatically. One button
  checks the connection so you're never guessing.

## Preview

### Health dashboard — everything at a glance
*Green means good. If something needs attention, you'll know — and you'll know what to do about it.*

![Health dashboard](assets/img/screenshots/status.png)

### Plain-English problems, not scary errors
*When something's wrong, Aegis tells you what happened, why it matters, and the steps to fix it.*

![Friendly errors](assets/img/screenshots/status-error.png)

### Account management
*Create an account, set a GM level, or reset a password — just fill in the boxes.*

![Accounts](assets/img/screenshots/accounts.png)

### One-click backups
*Back up your whole server to one file. See the size and a sanity check when it's done.*

![Backup](assets/img/screenshots/backup.png)

### Safe restore
*Aegis takes a safety backup first and won't restore while your server is running.*

![Restore](assets/img/screenshots/restore.png)

## Download & Install

> ⚠️ The first release isn't out yet — this is how it'll work once it is. For now, see
> [Building from source](#building-from-source).

1. Go to the [**Releases**](https://github.com/timoinglin/aegis/releases) page and download the
   latest **`Aegis-Setup.exe`**.
2. Double-click it to install.
3. Open **Aegis** from your Start menu or desktop. Done!

That's the whole install — no command line, no extra programs, nothing to set up by hand.

### "Windows protected your PC"

The first time you run Aegis, Windows might show a blue **"Windows protected your PC"** box. That's
normal for a small free tool like this one (it just means we haven't paid for an expensive
certificate). It's safe to continue:

1. Click **More info**.
2. Click **Run anyway**.

You'll only need to do this once.

## First time you open it

Aegis walks you through a quick setup the first time:

1. It tries to **find your repack automatically**. If it can't, you just point it at your repack's
   `_Server` folder (the one with `mysql` inside).
2. It **tests the connection** to your database so you know it's working.
3. You're done — the Health dashboard turns green and you can start using the buttons.

Your settings are saved on your PC (in your `AppData` folder), so you only do this once.

## Keeping it up to date

Aegis can check for new versions from inside the app and update itself when you click the button —
no reinstalling, no hunting for downloads. *(Coming with the first release.)*

## Is it safe?

Aegis is built to be careful, because we know a wrong click on a server can ruin your day:

- 🛟 **Backups before anything risky.** Restoring always takes a fresh safety backup first, so you
  can always go back.
- ✋ **It asks before doing anything destructive** — you have to type a confirmation, and Aegis won't
  let you restore while your server is running.
- 🔒 **Your password is never shown or saved in plain text.** Account passwords are handled by your
  own server.
- 📋 **Everything is written to a log file** (in your `AppData\Aegis\logs` folder). If you ever need
  help, you can attach that log so it's easy to see what happened.

## Troubleshooting & Help

This is a free project maintained by one person, so support works best through **GitHub** rather
than DMs — that way the fix helps the next person too:

- The **Health dashboard** explains most problems and how to fix them, right inside the app.
- Still stuck? [**Open an issue**](https://github.com/timoinglin/aegis/issues) and attach your log
  file from `AppData\Aegis\logs` — that's the fastest way to get help.

## Building from source

For developers (you don't need this to *use* Aegis). Requires
[Node.js](https://nodejs.org) and the [Rust toolchain](https://rustup.rs).

```bash
git clone https://github.com/timoinglin/aegis.git
cd aegis
npm install
npm run app:dev      # run in dev mode
npm run app:build    # build a release installer
```

Aegis is built with [Tauri](https://tauri.app/) (Rust backend) + React + TypeScript + Tailwind +
shadcn/ui. It shells out to the repack's own bundled `mysql` / `mysqldump` binaries and talks to the
worldserver's Remote Access — it never bundles its own database tools.

## Support the Project

Aegis is **free and MIT-licensed**, and it always will be — no paywall, no locked features. But a
lot of evenings go into building it, testing it on a live realm, and helping people get set up.

If Aegis saved you time or a headache, a small tip keeps it going — new features, fixes, and support.
Every coffee is hugely appreciated and genuinely motivating. 💛

[![Support the project on Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/kneuma)

## License

Aegis is licensed under the [MIT License](LICENSE).

Aegis is an independent community tool. It is not affiliated with or endorsed by EmuCoach.
