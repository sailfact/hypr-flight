# Security Policy

`hypr-flight` is a solo hobby prototype (see `galakz-prototype-spec.md` §1) —
a feel test, not a production application or service. There's no server
component, no user accounts, and no data collection, so the realistic attack
surface is small: local binary execution, the WASM build served from a static
page, and the crates it depends on.

## Supported versions

There are no numbered releases yet. If/when tagged releases start, only the
latest tag receives fixes — this isn't a project with a maintenance branch.

| Version | Supported |
|---|---|
| `develop` (latest) | ✅ |
| anything older | ❌ |

## Reporting a vulnerability

If you find something that could actually cause harm — e.g. a memory-safety
issue triggerable by a malicious level file or crafted input, or a
dependency with a known CVE that's actually reachable here — please open it
as a **private** report rather than a public issue:

- Use GitHub's [private vulnerability reporting](../../security/advisories/new)
  for this repo, or
- Contact the maintainer directly (see profile).

Please don't open a public issue for anything you believe is exploitable.

For everything else — crashes, panics, incorrect physics, visual bugs — a
normal public [bug report](./ISSUE_TEMPLATE/bug_report.md) is the right place.

## Dependencies

Dependencies are `bevy` and `rand` (see spec §4). Running `cargo audit`
occasionally against `Cargo.lock` is reasonable practice but not currently
automated in CI.
