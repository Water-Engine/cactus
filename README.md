# Cactus
Cactus is a framework providing a gui, cli and a library for playing chess and developing utilities for it.  
Cactus is written in rust and uses [raylib](https://www.raylib.com/index.html).  

# Getting Started
To build and run cactus, install the rust toolchain and run:
```shell
git clone https://github.com/Water-Engine/cactus.git
cd cactus
just build-all 
```
You can also build specific components individually:
```shell
just build-cli
just build-gui
just build-lib
```

# Dependencies
- [cargo](https://github.com/rust-lang/cargo)
- [just](https://github.com/casey/just)

# Building Cactus
The project's build system uses cargo with just. Below is a list of targets with their aliases:

# Build Specific Targets
| **Recipe**  | Alias | Description                                                                       |
|:------------|:-----:|:----------------------------------------------------------------------------------|
| `build-cli` | bc    | Builds the `cactus-cli` package                                                   |
| `build-gui` | bg    | Builds the `cactus-gui` package                                                   |
| `build-lib` | bl    | Builds the `libcactus` library                                                    |
| `build-all` | ba    | Builds the all the project packages                                               |
| `run-cli`   | rc    | Compiles and runs the cli. You can optionally pass the arguments for `cactus-cli` |
| `run-gui`   | rg    | Compiles and opens the gui                                                        |
| `clean`     | cln   | Cleans all the build artifacts                                                    |
| `fmt`       |   -   | Formats the rust code using cargo fmt                                             |
| `fmt-check` | fc    | Checks the formatting of all files                                                |

`build-*` recipes use `cargo build` under the hood, hence you can pass any of cargo's arguments, by default no arguments are passed.
A useful one is `--release` or `-r` to build with release mode, since by default builds are in debug mode.
 
# Motivation

cactus is designed as a lightweight and performant chess framework, providing a flexible foundation for both developers and players.

- `cactus-cli` enables stress-testing of chess engines, offering multiple tournament formats and easy export of match results.
- `cactus-gui` allows you to play PvP, Bot vs Player, or Bot vs Bot matches, and provides tools for analyzing games.
- `libcactus` serves as the core library, powering both the CLI and GUI applications.
