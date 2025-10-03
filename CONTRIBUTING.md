# Contributing Guide
Thank you for your interest in contributing to `cactus`!  

Cactus is a Rust-based framework providing a GUI, CLI, and core library for playing chess and developing chess-related utilities. It is designed to be:
- Lightweight: minimal dependencies and efficient code.
- Performant: suitable for real-time chess play and engine testing.
- Flexible: provide a foundation for both developers and players.

We welcome contributions of all kinds - from bug fixes and new features to documentation, testing, and feedback.

## Ways to contribute

Contributions are welcome in many forms - You don’t need to write code to help:  
- Code: add features, fix bugs, optimize performance.
- Documentation: improve README, guides, examples, or inline docs.
- Testing: write unit/integration tests, help reproduce reported issues.
- Feedback: suggest improvements via [issues](https://github.com/Water-Engine/cactus/issues).
## Before you start
- Check open [issues](https://github.com/Water-Engine/cactus/issues) - especially those labeled `good first issue` or `help wanted`.
- For significant changes, please open an issue or discussion first to align with the project goals.
- Read this guide fully to understand our expectations.

## Getting Started
1. Fork this repo and clone it locally:
```shell
git clone https://github.com/water-engine/cactus
cd cactus
git checkout -b feature/my-change
```
2. Install the dependencies:
- [Rust Toolchain](https://rust-lang.org/tools/install/)
- [Just](https://github.com/casey/just)
3. Refer to the [README.md](./README.md) for detailed usage instructions. 

## PR Guidelines
- Use descriptive names for your changes, including branch names and commit messages and write clear, readable code.
- Link related issues, and clearly describe what you changed and why.
- Keep changes focused, add/update tests if relevant.
- Make sure your code is formatted with `rustfmt` and linted using `clippy`.
- Make sure the changes align with the goals of cactus.
- Ensure all tests pass locally before opening a PR.

**Out-of-Scope Contributions Include:**
- Adding heavyweight or complex third-party crates, like `clap` or prebuilt libraries.
- Features that duplicate functionality already available in Rust’s standard library.
- Code that prioritizes convenience over performance or simplicity.

If in doubt, open a discussion before starting work.

## Getting Help
We're especially welcoming to first-time contributors.
All of us were beginners once. If anything is unclear, don’t hesitate to ask for help.
