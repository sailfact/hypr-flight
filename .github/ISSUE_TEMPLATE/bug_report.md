---
name: Bug report
about: Something behaves incorrectly (crash, visual glitch, wrong physics, etc.)
title: "[BUG] "
labels: bug
assignees: ''
---

## Summary

A short description of what's wrong.

## Steps to reproduce

1.
2.
3.

## Expected behaviour

What you thought would happen.

## Actual behaviour

What actually happened. Screenshots, a screen recording, or a paste of the
on-screen debug readout (velocity/state/tuning values) are all useful here.

## Environment

- Branch / commit: `develop` @
- Build type: native / web (WASM)
- OS:
- Bevy version (check `Cargo.toml` if unsure):

## Relevant tuning values (if flight/physics related)

If this is a feel or collision bug, paste the current values from `tuning.rs`
or the debug overlay — most flight bugs trace back to a specific constant
rather than logic.

## Additional context

Anything else — does it only happen at high speed, only near walls, only after
boost, etc.
