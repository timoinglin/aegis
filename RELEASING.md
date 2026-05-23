# Releasing Aegis

The exact steps to cut a signed release that the in-app updater can pick up.

## 0. Things you need once (and only once)

1. **A real code-signing decision.** Aegis is built for self-signed distribution: end users see a
   one-time "Windows protected your PC → More info → Run anyway" prompt. If you ever buy an OV/EV
   cert, swap it in here. Until then, the placeholder icon set under `src-tauri/icons/` is fine;
   replace it with `npx tauri icon path\to\final-logo.png` when the final transparent shield art is
   ready.

2. **An updater signing keypair.** The dev key in `src-tauri/.keys/` is a throwaway — **regenerate
   it before the first public release** and never lose the private key after:
   ```powershell
   npx tauri signer generate -w src-tauri\.keys\aegis-updater.key -p "<a strong password>"
   ```
   Then update the **public key** in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey` to the
   contents of `aegis-updater.key.pub`. **The private key + password are what every future release
   signs with** — losing them means no user can update from this version.

3. **GitHub Actions secrets** (so future releases can build in CI without exposing the key):
   - `TAURI_SIGNING_PRIVATE_KEY` — contents of `aegis-updater.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the password you set above

## 1. Pre-release checklist

- [ ] `npm run dev` smoke-test the app once: setup wizard → Status all green → Server tab status is
  live → one backup runs → list addons populates.
- [ ] `npx tsc --noEmit` clean.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` clean.
- [ ] Versions match across `package.json`, `src-tauri/tauri.conf.json` and
  `src-tauri/Cargo.toml`. Bump them together (semver).
- [ ] `CHANGELOG.md` (when we have one) up to date.

## 2. Build the signed release artifacts

In a clean PowerShell with the signing creds in the environment:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content -Raw src-tauri\.keys\aegis-updater.key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<the password>"
npm run app:build
```

This produces, under `src-tauri\target\release\bundle\nsis\`:

- `Aegis_<ver>_x64-setup.exe` — the installer end users download. The Tauri v2 updater
  installs from this `.exe` directly (no separate zip payload).
- `Aegis_<ver>_x64-setup.exe.sig` — the signature the updater verifies.

Aegis's `bundle.createUpdaterArtifacts` is on, so the `.sig` is generated automatically as long as
the signing env vars are set.

## 3. Make `latest.json`

The Tauri updater reads this file from the GitHub release. Create it (or generate from the build
output) with the fields below — `signature` is the **contents** of the `.sig` file:

```json
{
  "version": "<ver>",
  "notes": "<release notes — short>",
  "pub_date": "<ISO 8601 timestamp>",
  "platforms": {
    "windows-x86_64": {
      "signature": "<contents of Aegis_<ver>_x64-setup.exe.sig>",
      "url": "https://github.com/timoinglin/aegis/releases/download/v<ver>/Aegis_<ver>_x64-setup.exe"
    }
  }
}
```

The endpoint Aegis polls is hard-coded in `tauri.conf.json`:

```
https://github.com/timoinglin/aegis/releases/latest/download/latest.json
```

…so as long as `latest.json` is attached to the release that GitHub considers "latest", the in-app
updater finds it.

## 4. Publish the release

```powershell
$ver = "0.1.0"  # match the version you built
gh release create "v$ver" `
  "src-tauri\target\release\bundle\nsis\Aegis_${ver}_x64-setup.exe" `
  "src-tauri\target\release\bundle\nsis\Aegis_${ver}_x64-setup.exe.sig" `
  ".\latest.json" `
  --title "Aegis v$ver" `
  --notes-file .\RELEASE_NOTES.md
```

(Or use the GitHub web UI — same result.)

## 5. Verify

1. Visit the [Releases page](https://github.com/timoinglin/aegis/releases) and confirm all three
   artifacts (`.exe`, `.exe.sig`, `latest.json`) are attached.
2. On a fresh user machine: download `Aegis-Setup.exe`, install, open Aegis → **About → Check for
   updates** should now say "You're on the latest version."
3. Bump version locally, run step 2 again, publish v0.1.1 → on the old machine, the in-app updater
   should now find the update.

## Notes

- **Don't force-push** the `latest.json` between releases. The updater caches by URL; users may
  end up downloading a stale signature.
- **Don't change the updater pubkey** unless you've coordinated a re-install for existing users —
  old versions will reject the new signature otherwise.
- The bundled add-on zips under `src-tauri/resources/addons/` make the installer larger but they
  ship offline — keep them at a sensible size. To skip bundling them for a slim build, drop the
  glob from `bundle.resources` in `tauri.conf.json`.
