# GCPV Lynx Export Generator

A tool to watch a `.pat` file and export `LYNX.EVT` and `races.json` files for skating competitions.

## Features
- Windows GUI for easy configuration.
- Watch folder/file for changes.
- Configurable broadcast interval.
- Automated processing of competition data.

## Getting Started

### Prerequisites
- Rust toolchain

### Running
```bash
cargo run --release
```

## Development
This project uses `eframe` and `egui` for the UI.
Built releases are available in the GitHub Actions artifacts.
