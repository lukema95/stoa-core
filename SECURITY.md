# Security Policy

[English](SECURITY.md) | [简体中文](SECURITY.zh.md)

Stoa is **alpha software and has not been audited**. Do not deploy it to mainnet or use it with real funds without an independent security review.

## Supported versions

| Version | Supported |
|---|---|
| `main` | ✅ |
| Latest release | ✅ |
| Older releases | ❌ |

Security fixes are applied to `main` and the latest release. Older tags may not receive fixes; upgrade to the latest release.

## Reporting a vulnerability

Please **do not open a public GitHub issue** for security reports.

Email `lukema95@gmail.com` with:

- A description of the vulnerability and its impact.
- The affected component (program, IDL, tests, or docs).
- Steps to reproduce, including cluster, program ID, and transaction signatures where relevant.
- A proof of concept, logs, or screenshots if available.
- Your contact details and whether you wish to be credited.

If you need an encrypted channel, say so in your first email and we will coordinate a key exchange.

We will:

1. Acknowledge receipt within 3 business days.
2. Provide a status update at least every 7 business days until resolution.
3. Coordinate disclosure with you before publishing details.

## Scope

In scope:

- The Anchor program in `programs/stoa/`.
- Account validation, PDA derivation, fee accounting, and curve math.
- The generated IDL and TypeScript types.
- Supply-chain or build issues in this repository.

Out of scope:

- Front-end or backend integrations maintained in other repositories.
- Denial of service through the Solana network itself.
- Issues that require a compromised admin or upgrade-authority key.
- Social engineering, phishing, or physical attacks.
- Findings that only affect the local test validator.

## Severity and response

We use the following rough targets, adjusted for exploitability and impact:

| Severity | Initial response | Fix target |
|---|---|---|
| Critical | 1 business day | 7 days |
| High | 3 business days | 14 days |
| Medium | 7 business days | 30 days |
| Low | 14 business days | Next release |

## Safe harbor

We will not pursue legal action or report researchers to authorities for good-faith security research that:

- Stays within the scope above.
- Avoids privacy violations, data destruction, and service disruption.
- Uses only accounts and funds you control.
- Is reported privately and given reasonable time to fix before disclosure.

## Audit status

The program has **not** completed a formal third-party audit. See [docs/09-security.md](docs/09-security.md) for the threat model, invariants, and known limitations.

## Bug bounty

There is currently no paid bug bounty. We are happy to credit responsible reporters in release notes and security advisories, with your permission.

## Disclosure policy

We follow coordinated disclosure:

1. Confirm and assess the report.
2. Develop and test a fix.
3. Prepare a release and advisory.
4. Publish after users have had a reasonable opportunity to upgrade.
5. Credit the reporter if desired.

Please do not publish details before the advisory is released.

## Contact

- Security reports: `lukema95@gmail.com`
- General questions: see [SUPPORT.md](SUPPORT.md)
- Community standards: see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
