/*
    cactus-cli: A CLI tool to run and manage chess engine tournaments
    Copyright (C) 2025 SoloMazer

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod args;
mod helper;
mod initializer;
mod runner;
mod utils;

use crate::args::argument_parser;

fn main() {
    argument_parser();
}
