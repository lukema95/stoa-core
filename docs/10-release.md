# 10 Release Process

[English](10-release.md) | [简体中文](10-release.zh.md)

This document describes how Stoa Core is versioned, built, verified, and published. It is written for maintainers.

## 1. Versioning

Stoa follows [Semantic Versioning](https://semver.org/):

- **MAJOR** — breaking changes to instructions, account layout, PDA seeds, or the program ID.
- **MINOR** — backward-compatible instructions, accounts, or features.
- **PATCH** — backward-compatible fixes, documentation, or build changes.

While the project is `0.x`, minor releases may include breaking changes. Every breaking change must be called out in `CHANGELOG.md` and the release notes.

The protocol version, IDL version, and client package version should move together.

## 2. Release history

| Version | Date | Notes |
|---|---|---|
| 0.1.0 | 2025-10-05 | Initial commit and first public release |

## 3. Release checklist

- [ ] `main` is green and up to date.
- [ ] `CHANGELOG.md` and `CHANGELOG.zh.md` are updated.
- [ ] Documentation is updated in English and Chinese; `./scripts/check-docs.sh` passes.
- [ ] `Cargo.toml` and `programs/stoa/Cargo.toml` versions are bumped.
- [ ] `package.json` version is bumped if client-facing artifacts change.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] `cargo test -p stoa` passes.
- [ ] `anchor build` succeeds on a clean checkout.
- [ ] `anchor test --validator legacy` passes.
- [ ] `./scripts/check-program-id.sh` passes.
- [ ] The program keypair or upgrade authority is available to the deployer.
- [ ] For mainnet: an independent audit is complete and its report is linked.

## 4. Build artifacts

```bash
anchor build
```

Release artifacts:

| Artifact | Path | Committed? |
|---|---|---|
| SBF program | `target/deploy/stoa.so` | No |
| IDL | `target/idl/stoa.json` | Optional |
| TypeScript types | `target/types/stoa.ts` | Optional |
| Program keypair | `target/deploy/stoa-keypair.json` | **Never** |

Generate checksums:

```bash
shasum -a 256 target/deploy/stoa.so target/idl/stoa.json target/types/stoa.ts
```

Attach the `.so`, IDL, types, and checksums to the GitHub release. Never attach the keypair.

## 5. Tagging

```bash
git tag -s v0.1.0 -m "Stoa Core v0.1.0"
git push origin v0.1.0
```

Use signed tags. The tag must point to the commit that produced the artifacts.

## 6. GitHub release

The release notes should include:

- A summary of changes.
- Breaking changes and upgrade notes.
- The program ID and cluster status.
- The exact toolchain versions (Rust, Solana, Anchor, Node).
- Artifact checksums.
- A link to the audit report, if available.
- A link to the full changelog.

## 7. IDL package

A versioned `@stoa/idl` package is planned. When it exists:

1. Build the IDL and types from the tagged commit.
2. Set the package version to match the program version.
3. Publish with provenance.
4. Document the program ID and cluster in the package README.
5. Keep the package and program versions in lockstep.

Until then, integrators should pin a release tag and regenerate artifacts from that tag.

## 8. Deployment

Follow [08-deployment.md](08-deployment.md). For a release:

1. Deploy the `.so` with the recorded upgrade authority.
2. Verify the program ID and data length.
3. Initialize only if this is a fresh deployment.
4. Run a smoke test: create a pool, buy, sell, claim.
5. Announce the deployment with the program ID and commit.

## 9. Security releases

For a security fix:

1. Coordinate privately with the reporter.
2. Prepare the fix and a regression test.
3. Build and verify the artifacts.
4. Release a patch version and advisory.
5. Credit the reporter if desired.
6. Publish details after users have had time to upgrade.

Do not include exploit details in the commit message before disclosure.

## 10. Rollback

Keep the previous `.so`, IDL, and commit hash. To roll back, rebuild the previous tag and deploy it with the same upgrade authority. Announce the rollback and any data implications.

## 11. Post-release

- Monitor the program and event indexers for anomalies.
- Update documentation if behavior differs from the release notes.
- Open follow-up issues for deferred work.
- Record the release in `CHANGELOG.md`.
