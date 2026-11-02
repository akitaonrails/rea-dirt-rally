# Changelog

All notable changes to the DiRT Rally launcher. Versions are git tags
(`vX.Y.Z`) matching the `dirtrally-launcher` crate version.

## [Unreleased]

### Added

- Scaffold from rea-launcher-sdk v0.3.3 (`scripts/new-launcher`): edition
  table and unknown-edition refusal, option table (`look`, `resolution`,
  `vsync` as examples), settings window, `--check`, `--no-window`, the
  Linux helper, packaging and CI.
- Editions: Feral's Linux port 1.0 (build 31946) is recognised and
  playable; the Windows v1.1 `drt.exe` is recognised and refused with the
  reason (it stops while loading under Wine). The game is found in the
  chosen folder or its only subfolder.
- `dirtrally-exec-read`: a small preload library that makes executable
  memory readable, fixing the Linux port's start-up crash on CPUs with
  memory protection keys (AMD Zen 3 and later). Verified on a stage run.
