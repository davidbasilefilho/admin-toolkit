# admin-toolkit

A Windows-only terminal tool for staging hostname, password, and domain changes.

## Why

At the time of building this program, I worked as an intern at the Itu City Hall (Prefeitura da Estância Turística de Itu). I had to perform system administration tasks — changing hostnames, resetting passwords, joining machines to the domain — all manually through different Windows interfaces. Each task required navigating separate settings dialogs, clicking through multiple screens, and repeating the same steps dozens of times a day.

This tool centralizes those repetitive operations into a single, keyboard-driven terminal interface. It was built for my own workflow, but it reflects a real need: the average municipal office still runs on aging hardware, and anything that reduces clicks and context-switching makes the daily grind a little more bearable.

## Why Rust

I chose Rust for three reasons that mattered in that environment:

**Performance.** The machines at City Hall weren't new. Some were pushing ten years of continuous operation. Rust's zero-cost abstractions and small runtime footprint meant the tool would launch instantly and stay responsive even under constraints that would choke a heavier runtime.

**High-level ergonomics with low-level control.** Rust gives you the expressiveness of a high-level language — pattern matching, iterators, Result handling — while still letting you tap into Windows APIs directly. I could call `netdom.exe`, manipulate the Windows registry, and work with the Win32 API without writing a single line of C or C++.

**Mature TUI ecosystem.** At the time I started, `ratatui` (formerly `tui-rs`) was already a robust, well-maintained library for building terminal user interfaces in Rust. It gave me keyboard handling, layout primitives, and styling without the overhead of a GUI framework — exactly what I needed for a tool that lives in the console.

There were personal reasons too: I wanted to learn Rust properly, and building something I actually needed was the best way to do it. The compiler caught my mistakes before they became runtime bugs, which was reassuring when I was still learning the language.

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

<details>
<summary>Development build</summary>

```bash
cargo build
```

Produces a debug binary at `target/debug/admin-toolkit.exe`.

</details>

<details>
<summary>Release build</summary>

```bash
cargo build --release
```

Produces an optimized binary at `target/release/admin-toolkit.exe`.

</details>

<details>
<summary>Run tests</summary>

```bash
cargo test
```

</details>

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