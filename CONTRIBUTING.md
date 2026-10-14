# Contributing

Thanks for your interest in nunc. Issues and pull requests are welcome, in English or German.
Participation is governed by the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Before you start

- Larger changes: open an issue first so the approach can be discussed.
- Read [`CLAUDE.md`](./CLAUDE.md) (architecture rules, BDD workflow, checks), [`PLAN.md`](./PLAN.md) and the decisions in
  [`docs/adr/`](./docs/adr/). Security problems go to [SECURITY.md](./SECURITY.md), not to the issue tracker.

## Workflow

1. Fork, then branch off `main` (`feat/…`, `fix/…`, `docs/…`).
2. **Behaviour first:** describe it as Gherkin scenarios (English) in `features/`, then implement
   (`nunc-core` → `nunc-store`/`nunc-sync` → command → UI). Scenarios are specification and test at once.
3. Run the checks:

   ```bash
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   npm run typecheck && npm run lint && npm run test
   npm run docs:report
   ```

   After changes to commands or specta types, run `cargo test -p nunc export_bindings` and commit `src/bindings.ts`.
4. Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `test:`, `chore:` …).
5. Open a pull request against `main`. CI (`checks`) must be green. Visible texts go into `src/i18n/de.ts` and `en.ts`.

## Rules worth knowing

- No business logic in `src-tauri` commands or React components; it belongs in `nunc-core`.
- Time comes from the injected `Clock`, not from `SystemTime::now()` / `Date.now()`.
- Never log or store secrets (tokens, passwords) outside the `SecretStore`.
- Do not change migrations 001–005; schema changes need a new, additive migration.

## License

By contributing you agree that your contribution is licensed under
[GPL-3.0-or-later](./LICENSE), the license of the project.
