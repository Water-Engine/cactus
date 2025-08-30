# Contributing Overview

This project's source code and build system is designed such that it can be run on any major platform assuming you have the correct tools installed. To confirm cross-platform behavior, GitHub Actions runs the project's build and test system on Windows, Linux, and macOS. This project is compiled using the GNU Compiler Collection's g++ compiler, and formatting is done through clang-format. Building this repository on your own should be as simple as running `make`. Please follow the project's formatting guides, and call `make fmt` on code that you wish to contribute. Please do not push AI-generated code. This project should be a learning experience, not a copy-paste speedrun. Additional learning resources can be found in [READING.md](READING.md). All code should be submitted to main via pull request, and your username can be added to `AUTHORS.md` upon merge.

Never commit code to the `loc` branch. It is unprotected but volatile. GH actions will reset this branch every time main is updated, so anything that shouldn't be there will be lost to the void...

# Formatting
`cargo fmt` is used for formatting on this side of the project. No custom formatting rules apply to written Rust code. The only requirement is that it aligns with the standard enforcing by cargo.

# Testing
There are no tests for the GUI. This should be changed eventually, especially as the App grows into something more usable.
