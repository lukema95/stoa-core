# Governance

[English](GOVERNANCE.md) | [简体中文](GOVERNANCE.zh.md)

Stoa is a small, security-sensitive open-source project. Governance is intentionally lightweight, but changes that affect user funds or the deployed program require explicit review.

## Roles

### Contributors

Anyone who reports issues, improves documentation, adds tests, or submits code. Contributors participate in discussions and pull requests.

### Maintainers

Maintainers have merge rights and are responsible for releases, security response, and the long-term direction of the project. The current list is in [MAINTAINERS.md](MAINTAINERS.md).

### Security owner

A maintainer designated to coordinate vulnerability reports, security reviews, and advisories. The security owner may impose an embargo on details until a fix is released.

## Decision making

- **Routine changes** (docs, tests, small fixes): lazy consensus. A maintainer approves and merges.
- **Protocol changes** (curve math, fees, account layout, instructions): at least one maintainer review plus a 72-hour comment period. If there is no consensus, maintainers discuss and decide by simple majority.
- **Security fixes**: the security owner coordinates; details may be private until disclosure.
- **Program ID, upgrade authority, or deployment changes**: maintainer consensus and a written record in the pull request or release notes.
- **License or governance changes**: maintainer consensus and a public discussion period of at least 7 days.

## Releases

Releases follow [Semantic Versioning](https://semver.org/) and the process in [docs/10-release.md](docs/10-release.md). Maintainers may publish patch releases for security fixes with a shortened review period.

## Upgrades and the program ID

- The canonical program ID is recorded in `Anchor.toml`, `lib.rs`, and the IDL.
- Changing the program ID requires a maintainer decision and a compatibility note for integrators.
- The upgrade authority should be a multisig controlled by multiple maintainers. A single maintainer must not be able to upgrade the program unilaterally.
- Upgrades that change account layout must include an upgrade plan and a compatibility note.

## Becoming a maintainer

A contributor may be invited to become a maintainer after sustained, high-quality contributions in code, reviews, documentation, or security. Maintainers are expected to:

- Follow the Code of Conduct.
- Review changes carefully and promptly.
- Never merge unreviewed security-sensitive changes.
- Help with releases and incident response.
- Disclose conflicts of interest.

New maintainers are approved by existing maintainers. Maintainers who are inactive for 6 months may be moved to emeritus status after a check-in.

## Conflicts of interest

Maintainers must disclose financial or employment interests that could affect decisions about the protocol, deployments, or integrations. They should recuse themselves from decisions where they have a direct conflict.

## Amendments

Governance changes are proposed via pull request, discussed publicly for at least 7 days, and approved by maintainer consensus.
