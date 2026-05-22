# Icons

This folder is intentionally empty for now — the skeleton ships **without a final
icon** so we're not blocked on branding (per the v0.1 plan).

When the final transparent shield PNG is ready, generate the full set from it:

```powershell
# from the repo root, after `npm install`
npx tauri icon path\to\aegis_shield.png
```

That command writes `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.ico`,
`icon.icns` and the Windows Store logos here — matching the `bundle.icon` list in
`tauri.conf.json`. Until then, `tauri build` / `tauri dev` will ask for these files.
