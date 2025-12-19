# Phoenix AGI: Sola Edition Windows Build Guide

This guide explains how to build and package the Phoenix AGI: Sola Edition application for Windows.

## Prerequisites

- Windows 10 or 11
- Rust toolchain (install via rustup)
- Node.js (for frontend building)
- Inno Setup 6 (for creating the installer)

## Build Scripts

The following scripts are provided to build Phoenix AGI:

### 1. `launcher.cmd`

A simple launcher script that starts `phoenix-web.exe` and opens a browser to http://127.0.0.1:8888.

### 2. `build_windows.cmd`

This script builds both the frontend and backend, then creates a staging directory with all required assets:

- Builds the frontend using `scripts/build_frontend.cmd`
- Builds the Rust backend with `cargo build --release --bin phoenix-web`
- Creates a clean staging folder with all required assets
- Sets up a default `.env` file

### 3. `installer.iss`

Inno Setup script that creates the PAGI-SolaSetup.exe installer:

- Installs to `%LOCALAPPDATA%\Phoenix` to avoid permission issues
- Creates start menu and optional desktop shortcuts
- Includes an uninstaller
- Detects and closes any running Sola instances

### 4. `build_installer.cmd`

This script brings everything together:

- Runs `build_windows.cmd` to build the app and create the staging directory
- Compiles the installer using Inno Setup

## Building the Installer

To build the complete Windows installer, simply run:

```
build_installer.cmd
```

This will:
1. Build the frontend and backend
2. Create the staging directory with all needed files
3. Compile the installer using Inno Setup
4. Produce `PAGI-SolaSetup.exe` in the current directory

## Installation and Usage

After installation:

1. Phoenix AGI: Sola Edition will be installed to `%LOCALAPPDATA%\Phoenix`
2. The application can be launched from the Start menu or desktop shortcut
3. User settings (API keys, etc.) are stored in the `.env` file in the installation directory

## Configuration

After installation, users can configure Phoenix AGI through the Settings panel in the UI, which will modify the `.env` file. The following settings are available:

- OpenRouter API Key
- USER_NAME (for the primary address name in relational context)
- USER_PREFERRED_ALIAS (what Sola calls you)