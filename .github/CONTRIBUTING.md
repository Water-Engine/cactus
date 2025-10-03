# Contributing Guide
Thank you for your interest in contributing to `cactus`!  

Cactus is a Rust-based framework providing a GUI, CLI, and core library for playing chess and developing chess-related utilities. It is designed to be:
- Lightweight: minimal dependencies and efficient code.
- Performant: suitable for real-time chess play and engine testing.
- Flexible: provide a foundation for both developers and players.

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

## Building cactus from source

To build and run cactus, install the rust toolchain and run:
```shell
git clone https://github.com/Water-Engine/cactus.git
cd cactus
git switch rewrite
just build-all 
```
You can also build specific components individually:
```shell
just build-cli
just build-gui
just build-lib
```

### Dependencies
- [cargo](https://github.com/rust-lang/cargo)
- [just](https://github.com/casey/just)

### Building Cactus
The project's build system uses cargo with just. Below is a list of targets with their aliases:

#### Build Specific Targets
| **Recipe**  | Alias | Description                                                                       |
|:------------|:-----:|:----------------------------------------------------------------------------------|
| `build-cli` | bc    | Builds the `cactus-cli` package                                                   |
| `build-gui` | bg    | Builds the `cactus-gui` package                                                   |
| `build-lib` | bl    | Builds the `libcactus` library                                                    |
| `build-all` | ba    | Builds the all the project packages                                               |
| `run-cli`   | rc    | Compiles and runs the cli. You can optionally pass the commands for `cactus-cli`  |
| `run-gui`   | rg    | Compiles and opens the gui                                                        |
| `clean`     | cln   | Cleans all the build artifacts                                                    |
| `fmt`       |   -   | Formats the rust code using cargo fmt                                             |
| `fmt-check` | fc    | Checks the formatting of all files                                                |

`build-*` recipes use `cargo build` under the hood, hence you can pass any of cargo's arguments, by default no arguments
are passed.
A useful one is `--release` or `-r` to build with release mode, since by default builds are in debug mode.

> [!IMPORTANT]
> Note that when using the `run-*` commands, a temporary `test` folder, relative to the justfile, will be created to store
all the necessary configs.
> This is to avoid polluting the project space, and ambiguity in execution of justfile compared to cargo.

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
