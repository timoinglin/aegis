# Bundled add-ons

Aegis can install these add-ons into the user's `Interface\AddOns` folder with one
click. They're bundled into the app (via `tauri.conf.json` → `bundle.resources`).

## How to add one

For each add-on listed in `catalog.json`, drop two files **in this folder**, named
by the entry's `id`:

- `<id>.zip` — the add-on download. The zip should contain the add-on's folder(s)
  at its **root** (e.g. `Bagnon/…`), like a normal CurseForge / legacy-wow zip.
  Aegis extracts it straight into `Interface\AddOns`.
- `<id>.png` — a thumbnail/screenshot shown in the Add-ons list.

Example for DBM: `dbm.zip` + `dbm.png`.

The `folder` field in `catalog.json` is the AddOns subfolder the zip creates; Aegis
uses it to tell whether the add-on is already installed. If a zip uses a different
top folder name, fix that field.

## Notes

- These are third-party add-ons — keep them reasonably up to date, and only ship
  ones whose license permits redistribution.
- Entries with no matching `.zip` present are simply hidden in the app, so it's
  fine to fill these in over time.
