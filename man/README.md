# ReedCMS Man Pages

This directory contains man page documentation for the `reed` CLI in Markdown format (`.ronn` files).

## Building Man Pages

Man pages are built using `ronn-ng`, a modern Ruby-based tool for generating Unix man pages from Markdown.

### Installation

```bash
# Install ronn-ng
gem install ronn-ng
```

### Build Process

```bash
# Generate man page from .ronn source
ronn --roff --pipe man/reed.1.ronn > man/reed.1

# View generated man page
man man/reed.1

# Install to system
sudo cp man/reed.1 /usr/share/man/man1/
sudo mandb  # Update man database
```

### Installation Paths

**Debian/Ubuntu:**
```bash
/usr/share/man/man1/reed*.1
```

**Homebrew (macOS):**
```bash
$(brew --prefix)/share/man/man1/
```

**Custom MANPATH:**
```bash
export MANPATH=/path/to/reedcms/man:$MANPATH
```

## File Structure

```
man/
├── README.md           # This file
├── reed.1.ronn         # Main reed CLI man page (Markdown source)
└── reed.1              # Generated groff man page (binary format)
```

## Generating Documentation

The man pages are generated from the main `.ronn` file:

- `reed.1.ronn` → `reed.1` (main CLI documentation)

## Viewing Man Pages

```bash
# View local man page (without installation)
man ./man/reed.1

# View installed man page
man reed

# Search for specific command
man reed | grep -A5 "set:text"
```

## Format Specification

Man pages use `ronn` format, which is Markdown with man page extensions:

- `##` - Section headers (SYNOPSIS, DESCRIPTION, etc.)
- `` ` `` - Code/command formatting
- `*` - List items
- `  * term:` - Definition lists

## License

Copyright 2025 Vivian Voss. Licensed under the Apache License, Version 2.0.
