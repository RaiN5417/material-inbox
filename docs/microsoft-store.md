# MSIX packaging & Microsoft Store submission

Status: local MSIX build pipeline works and produces a valid (self-signable)
package. Nothing has been submitted to Partner Center yet — the placeholder
identity below must be replaced with real reserved values first.

## How it's built

Tauri's own bundler doesn't produce MSIX (only NSIS/MSI/portable). This repo
uses the third-party [`@choochmeque/tauri-windows-bundle`](https://github.com/Choochmeque/tauri-windows-bundle)
CLI (`apps/desktop/devDependencies`) for that, driven by:

- [`apps/desktop/src-tauri/gen/windows/bundle.config.json`](../apps/desktop/src-tauri/gen/windows/bundle.config.json) — publisher, capabilities, extensions, signing. Tracked in git (unlike the rest of `gen/`, which is regenerated Tauri/mobile scaffolding).
- [`apps/desktop/src-tauri/gen/windows/AppxManifest.xml.template`](../apps/desktop/src-tauri/gen/windows/AppxManifest.xml.template) — the manifest template; also tracked.
- `apps/desktop/src-tauri/gen/windows/Assets/` — MSIX tile logos, copied from `src-tauri/icons/` (placeholder branding — see `icons/README.md`).
- [`apps/desktop/scripts/build-msix.mjs`](../apps/desktop/scripts/build-msix.mjs) — thin wrapper invoked by the `tauri:windows:build` npm script.

Build it with:

```bash
cd apps/desktop
pnpm run tauri:windows:build
```

Output: `target/msix/AssetPile｜材栈_<version>.0.msixbundle` (workspace-root
`target/`, same as every other Cargo output in this repo).

### Why the wrapper script exists

Two things this repo's setup runs into that the packaging tool doesn't
handle on its own:

1. **Workspace target dir.** This is a Cargo workspace, so `cargo build`
   always writes to the workspace-root `target/`, not
   `apps/desktop/src-tauri/target/`. The tool needs `CARGO_TARGET_DIR` set to
   know that — but as a *relative* value it resolves inconsistently, because
   Tauri's own cargo invocation runs one directory deeper
   (`src-tauri/`) than the packaging tool's own process (`apps/desktop/`).
   `build-msix.mjs` computes an absolute path instead, which both processes
   resolve identically regardless of cwd.

2. **Executable name.** The tool assumes the compiled binary is named after
   `productName` with spaces stripped (`AssetPile.exe`, approximately — see
   below), with no config override. This project's actual Cargo binary is
   `assetpile.exe` (from `[package] name` in `Cargo.toml`), which the
   NSIS/MSI/portable pipeline already depends on
   (`.github/workflows/release.yml`, autostart registration, etc.) —
   renaming it project-wide would ripple through all of that. Instead,
   `src-tauri/Cargo.toml` declares a second `[[bin]]` target (`AssetPile`,
   same `src/main.rs`) purely so the MSIX tool finds the name it expects;
   the release pipeline keeps using `assetpile.exe` unchanged. Note
   `productName` is actually `"AssetPile｜材栈"` (with the CJK/pipe suffix)
   — this second bin's name is only an ASCII approximation and hasn't been
   verified against what the tool literally derives; check before relying
   on the MSIX pipeline.

## Signing

`bundle.config.json`'s `signing.pfx`/`pfxPassword` (or `MSIX_PFX_PASSWORD`
env var) are for **local sideload testing only** — Microsoft re-signs the
package itself when a Store submission is ingested, so no certificate needs
to be purchased for the submission to go through. To test-install a local
build, you'd sign with a self-signed cert whose Subject exactly matches
`bundle.config.json`'s `publisher` field, then trust it and enable Developer
Mode — that's a local machine-trust change, so it's left as a manual step
rather than something automated here.

## What's still a placeholder

- **Identity Name**: `dev.assetpile.app` (from `tauri.conf.json`'s
  `identifier`) is a placeholder. Once you reserve the app name in
  [Partner Center](https://partner.microsoft.com/dashboard), Microsoft
  assigns the real Package/Identity Name and Publisher CN. Override them for
  the Windows build only — without touching the shared `tauri.conf.json`
  the NSIS/MSI pipeline also reads — via a new
  `apps/desktop/src-tauri/tauri.windows.conf.json`:
  ```json
  { "identifier": "<reserved-identity-name>" }
  ```
  and update `bundle.config.json`'s `publisher` to the exact CN Partner
  Center gives you (must match character-for-character, or `MakeAppx`
  rejects it).
- **Branding**: the MSIX tile assets are the same placeholder icon set noted
  in `icons/README.md` — swap before submitting.
- **WebView2 runtime**: the NSIS installer bootstraps WebView2 at install
  time; an MSIX package can't run arbitrary install-time logic like that.
  This build currently assumes the Evergreen WebView2 Runtime is already on
  the machine (true by default on Windows 11, and on Windows 10 once it's
  had the runtime pushed via Windows Update — which is the common case but
  not guaranteed). If that turns out to be a real problem during
  certification, the fix is bundling the Fixed Version runtime as an app
  dependency; not done here.

## Capabilities already set

- `internetClient` — the update-check calls the GitHub Releases API from the
  frontend (`apps/desktop/src/lib/update.ts`).
- `runFullTrust` (added automatically by the packaging tool) — this package
  runs as a Desktop Bridge full-trust app, not in an AppContainer, so normal
  Win32 file access (watching/moving files anywhere in the user's Downloads
  folder) works without the restricted `broadFileSystemAccess` capability
  that sandboxed UWP apps would need.

## Before submitting

- Enroll as a developer in Partner Center (individual or company).
- Reserve the app name, then fill in the real identity (see above).
- Prepare Store listing assets (screenshots, description) and answer the age
  rating questionnaire.
- Since the app calls out to the internet (GitHub API for update checks),
  expect Partner Center to ask for a privacy policy URL even though no user
  data leaves the device otherwise.
