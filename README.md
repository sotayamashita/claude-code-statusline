# Beacon

[![CI](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml/badge.svg)](https://github.com/sotayamashita/beacon/actions/workflows/ci.yml)

## Overview
<!-- LLM Instructions: Update @specs/project.md when you change this section -->

**Beacon** is a lightweight, high-performance status line generator written in Rust, designed for AI-powered development environments. It provides a starship-like configuration experience.

## Motivation

Beacon exists to provide a fast, embeddable status line specifically tailored for Claude Code. I deeply respect and admire Starship for setting the bar on modular, configurable prompts across shells. However, Starship is intentionally delivered as a standalone CLI and does not expose a stable, supported Rust library API or a general plugin API for embedding its internals into other binaries. Because I need programmatic composition with JSON input and tight integration within AI-driven editor workflows, I built Beacon as a small Rust library/binary that borrows Starship’s proven ideas (modules, formatting, styling) while remaining easy to integrate as part of a larger toolchain.

## Acknowledgments

This project was inspired by [Starship](https://starship.rs/), the excellent cross-shell prompt. I've adapted its modular architecture for Claude Code's statusline.
