<p align="right"><a href="privacy-policy.md">简体中文</a> | English</p>

# Privacy Policy — AssetPile｜材栈

**Last updated: 2026-09-15**

AssetPile｜材栈 ("the App") is a local-first Windows utility for organizing downloaded files. This policy describes what data the App accesses, stores, and transmits.

## Summary

- The App does not collect, store, or transmit any personal information.
- No account or sign-in is required or supported.
- All app data (tracked files, tags, groups, operation history, settings) is stored only in a local SQLite database on your own device — in the App's local data directory, or a `data` folder next to the executable in the portable build. Nothing is uploaded anywhere.
- The only network request the App makes is a request to GitHub's public Releases API to check whether a newer version is available — see "Update checks" below.

## Local file access

The App monitors folders you choose (by default, your Downloads folder) to detect newly downloaded files, and performs file operations (move, rename, tag, delete) that you request, on your own device. This activity is entirely local: file names, paths, and any tags you apply are recorded only in the App's local database and never leave your device.

## Update checks (the App's only network activity)

When you open the App's Settings page (if "Automatically check for updates" is enabled, which is the default) or click "Check for Updates," the App sends a single, unauthenticated HTTP GET request to:

```
https://api.github.com/repos/RaiN5417/AssetPile/releases/latest
```

This request does not include your name, file names, file contents, device identifiers, or any other information from your device — it's the same request any browser would make to view that page. GitHub may log standard web request metadata (such as IP address and timestamp) under its own privacy practices; beyond reading a version number and a release URL from the response, the App itself does not access, store, or process any other data from this request. You can turn this check off entirely in Settings.

## No analytics, telemetry, or advertising

The App contains no analytics SDKs, crash-reporting services, telemetry, or advertising of any kind.

## Third parties

The only third-party service the App communicates with is GitHub (for the update check described above). The App does not share any information with any other party.

## Children's privacy

The App does not knowingly collect information from anyone, including children, because it does not collect personal information at all.

## Data retention and deletion

All App data stays on your device, under your control. Uninstalling the App does not automatically delete its local database; you can remove it yourself from the App's data folder, or use a reset option in the App's settings if one is offered.

## Changes to this policy

If this policy changes, the updated version will be published at this same URL, with a revised "Last updated" date.

## Contact

Questions about this policy or the App can be raised via a GitHub issue on the project's repository: <https://github.com/RaiN5417/AssetPile/issues>
