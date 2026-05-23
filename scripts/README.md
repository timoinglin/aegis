# `scripts/`

Dev-only helpers for tasks the build itself doesn't cover.

## `capture-aegis.ps1`

Capture the current Aegis window to a PNG. Uses `PrintWindow` with
`PW_RENDERFULLCONTENT` (flag 2) so it works on the WebView2/Chromium
surface — a plain screen-grab would either miss the window contents or
catch whatever's overlapping it.

```powershell
.\scripts\capture-aegis.ps1 -OutFile assets\img\screenshots\status.png
```

Aegis must be running. The script grabs whatever tab is currently shown.

## `regen-screenshots.ps1`

Drives the running Aegis window through every main tab and saves a fresh
PNG per tab to `assets\img\screenshots\`. Useful before cutting a release.

```powershell
# 1. Start Aegis (dev build or installed):
npm run app:dev
# (in another terminal, once the window has opened)

# 2. Regenerate:
.\scripts\regen-screenshots.ps1
```

Internals: it forces the window to a known size + position (1000x720 at
40,40), then sends mouse clicks into the sidebar by client-area
coordinates and captures after each. Coordinates assume the default
layout from `src/App.tsx`.

If you change the sidebar (add/remove a nav item or change padding),
update the `$tabs` table at the top of the script.
