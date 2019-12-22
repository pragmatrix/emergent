//! A package the implements terminal related features that are used in the emergent project.
//!
//! Most of it was taken from the
//! Alacritty project
//!   alacritty_terminal/src
//!   commit 629ea247cdccc33c6df0037f357ba9be48c7178d
//!   and slightly modified.
//!
//! Details about the modifications are listed in the individual source files.

//! Mandatory to get rid of a number of warnings:
#![allow(dead_code)]

#[macro_use]
extern crate log;

mod ansi;
pub mod color_schemes;
pub mod config;
mod index;
pub mod term;
pub mod text_attributor;
