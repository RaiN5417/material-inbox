## Goal

What does this PR do, and why?

## Scope

Files / crates / packages touched:

## Design decision

Anything non-obvious about the approach (see the Agent Task Template in
docs/download_inbox_product_technical_spec_v0.2.md section 61 for the shape this should take).

## Tests

- [ ] Unit tests
- [ ] Integration tests (if this touches watcher / file-operations / storage)

## Checklist

- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all pass
- [ ] `pnpm lint`, `pnpm typecheck`, `pnpm build` all pass
- [ ] No new dependency without a one-line justification (spec section 48)
- [ ] Doesn't break Undo / Local-first / Safe-by-default guarantees
- [ ] Docs updated if behavior or architecture changed
