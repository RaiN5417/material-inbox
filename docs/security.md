# Security & privacy

Defaults (spec section 34):

```text
No account · No cloud · No telemetry · No AI · No file upload
Network = 0 by default
```

## What the local DB holds

Filenames, local paths, and (from v0.2 on) source URLs / page titles when the
browser extension is used. All of it stays in the per-user app data
directory (`app.path().app_data_dir()`), never uploaded.

## Logging

`tracing`, release default `INFO`/`WARN`. Debug logs may include paths but
never file contents; anything bundled for a bug report must be
redactable — don't dump raw URL query strings into logs.

## File operation safety

Every mutating operation goes through `file-operations` and the
preflight → log → execute → verify → commit pipeline (spec section 35).
No other crate calls `std::fs::remove_file`/`rename` on a user path
directly — enforce this in review, not just by convention.

Report a vulnerability by opening a private security advisory on this repo
rather than a public issue.
