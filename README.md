# admin-toolkit

Windows-only terminal tool for staging hostname, password, and domain changes.

## What it does

- Shows current hostname and domain.
- Lets you stage three independent actions:
  - change hostname
  - change password for `Prefeitura`
  - change domain to `itu.local`
- Requires elevation.
- Applies changes only after a confirm screen.

## Controls

- `↑` / `↓` move between actions
- `Space` toggle selected action
- `e` edit hostname or password target
- `Enter` confirm or apply
- `Esc` go back
- `q` quit

## Build and test

```bash
cargo test
```

## Release

Tagged releases use the GitHub Actions workflow in `.github/workflows/release.yml`.

- Tag format: `v*`
- Builds Windows binaries for:
  - `i686-pc-windows-msvc`
  - `x86_64-pc-windows-msvc`
  - `aarch64-pc-windows-msvc`
- Publishes the binaries to the GitHub Release page for that tag.

## CI

GitHub Actions test workflow: `.github/workflows/test.yml`
