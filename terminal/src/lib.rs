//! A package the implements terminal related features that are used in the emergent project.
//!
//! Most of it was taken from the
//! Alacritty project
//!   alacritty_terminal/src
//!   commit 44037fa42aa80002ce54f0a8e4a6203e3e12aaf5
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
