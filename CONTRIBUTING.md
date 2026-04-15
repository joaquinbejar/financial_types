# Contributing to financial_types

Thank you for considering a contribution. This crate is the base layer
that downstream financial libraries (`option_type`, `positive`,
`expiration_date`, and others) depend on, so changes are held to a
strict semver and quality bar.

## Before you start

- Open or comment on an issue describing the change. Breaking changes
  require explicit discussion before any code is written.
- Read the [Code of Conduct](./CODE_OF_CONDUCT.md).

## Workflow

1. Fork the repository and create a feature branch from `main`:
   `git checkout -b issue-NN-short-description`.
2. Make focused commits — one logical change per commit.
3. Add or update tests covering every new variant, helper, or trait
   impl.
4. Add an entry under `[Unreleased]` in [`CHANGELOG.md`](./CHANGELOG.md)
   for any public-API change.
5. Run the full local check suite (see below) before pushing.
6. Open a PR against `main`. Link the issue with `Closes #NN`.

## Local checks

Run the same checks CI runs:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
cargo build --no-default-features
cargo doc --no-deps --all-features
```

Or use the project shortcut:

```bash
make pre-push
```

## Dependency policy

`cargo-deny` enforces the supply-chain rules (`deny.toml`):

- **Licenses**: only MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC,
  Unicode-DFS-2016, Unicode-3.0, Zlib, MPL-2.0, CC0-1.0. Anything else
  needs explicit approval and a `deny.toml` exception.
- **Banned deps**: never depend on `option_type`, `positive`, or
  `expiration_date` — they depend on us.
- **Sources**: only the official crates.io registry is allowed.
- **Advisories**: yanked versions and known vulnerabilities fail CI.

Run locally with `cargo deny check` before pushing dependency changes.

## Mutation testing

`cargo-mutants` runs in CI on every PR (and a full schedule on
Sundays). Unkilled mutants mean the test suite cannot distinguish the
intended behavior from a wrong implementation — fix by adding a test,
not by `#[mutants::skip]`. Reproduce locally:

```bash
cargo install cargo-mutants
cargo mutants --no-shuffle -j 2 --all-features --timeout 120
```

The report lands in `mutants.out/`.

## Releases

Releases are automated via [`release-plz`](https://release-plz.ieni.dev/).
Every push to `main` runs two jobs:

1. **release-pr** — if the release PR doesn't exist yet, opens one that
   bumps `Cargo.toml`, refreshes `CHANGELOG.md` from Conventional
   Commits, and prepares the release notes.
2. **release** — if a tagged version is not yet on crates.io, tags
   `vX.Y.Z`, creates the GitHub release, and runs `cargo publish`.

To cut a release: land changes via conventional-commit messages
(`feat:`, `fix:`, `feat!:` for breaking, etc.), then merge the
release PR that `release-plz` opens. Manual releases are discouraged;
use this flow for every version bump.

Secret `CARGO_REGISTRY_TOKEN` must be set at the repository level
(maintainer action). Without it, `release-plz release` will open the
GitHub release but skip `cargo publish`.

## Benchmarks

`benches/enums.rs` is a criterion suite covering every hot-path helper.
Run locally:

```bash
cargo bench
```

HTML reports land under `target/criterion/`. Benchmarks are not run in
regular CI — invoke on demand when changing anything performance-
sensitive (`as_str`, `Display`, `FromStr`, `TryFrom`).

## Coding standards

- Preserve `#[repr(u8)]` on every public enum. Assert the 1-byte size
  in tests.
- Use explicit discriminant values on every new variant (`= 0`, ...)
  to lock wire compatibility.
- `#[must_use]` on every pure helper, `#[inline]` on small hot
  helpers, `const fn` where possible.
- No `.unwrap()`, `.expect()`, `panic!`, or unchecked indexing in
  `src/` outside `#[cfg(test)]`.
- No logging in `src/` (no `println!`, `eprintln!`, `dbg!`, `log`).
- No new dependencies without explicit approval. Never depend on
  `option_type`, `positive`, or `expiration_date` (they depend on us).
- All comments, commit messages, and PR descriptions in English.

## Semver discipline

- Adding a variant to a non-`#[non_exhaustive]` enum is a **breaking**
  change.
- Renaming any variant string (which changes the JSON wire format) is
  a **breaking** change.
- Changing a discriminant value is a **breaking** change.
- Removing or renaming a public item is a **breaking** change.

When in doubt, run `cargo semver-checks check-release` locally.

## Reporting bugs

Open a GitHub issue with:
- A minimal reproduction (Rust playground link if possible).
- Expected vs. actual behavior.
- Crate version + Rust toolchain version.

## Contact

- Maintainer: Joaquín Béjar García
- Email: jb@taunais.com
- Telegram: [@joaquin_bejar](https://t.me/joaquin_bejar)
