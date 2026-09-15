# Icons

`source-icon.png` and the generated set (`icon.ico`, `32x32.png`, `128x128.png`,
`128x128@2x.png`, `icon.png`, `icon.icns`, `Square*Logo.png`, `StoreLogo.png`)
are AssetPile｜材栈's real branding, generated from the official logo with
`pnpm tauri icon`.

To regenerate from a new 1024x1024 source PNG:

```bash
pnpm tauri icon path/to/new-source-icon.png
```

That overwrites everything `tauri.conf.json` points at. `pnpm tauri icon`
also emits Android/iOS icon sets by default — this project has no mobile
target (see spec non-goals), so delete `icons/android/` and `icons/ios/`
after regenerating.
