[**日本語版README**](https://github.com/taichi765/TsukuyomiDMX/blob/main/README_ja_JP.md)
## Overview
Tsukuyomi is a DMX Lighting Controller written in Rust.  Work in progress.
## Motivation
Existing lighting controllers have trade-offs:
- Some are powerful but have steep learning curves.
- Others require proprietary hardware or licenses.
- Open-source solutions can be unstable or lack polish.

I started Tsukuyomi to build a Rust-native controller focused on usability, stability, and modern UI.
## Building
**Prerequisites**: [Rust](https://rust-lang.org/ja/tools/install/)
&nbsp;  
Currently we depends on a forked version of `slint`, so you need to clone it:
```shell
git clone https://github.com/taichi765/tsukuyomi-rs.git
git clone https://github.com/taichi765/slint.git
cd tsukuyomi-rs/
cargo run
```
## Roadmap
- [ ] DMX/ArtNet output
- [ ] Fixture library
- [ ] MIDI control
- [ ] 3D preview
## Contributing
Contributions are welcome! Please open issues for bugs/feature requests and create PRs for code changes.
