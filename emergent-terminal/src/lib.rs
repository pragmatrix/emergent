//! A crate the implements terminal related features that are used in the emergent project.
//!
//! Most of it was taken from the
//! Alacritty project rev 629ea247cdccc33c6df0037f357ba9be48c7178d and slightly modified.
//! Details about the modifications are listed in the individual source files.

#[macro_use]
extern crate log;

mod ansi;
mod config;
mod index;
mod term;
