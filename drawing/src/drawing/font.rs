use crate::scalar;
use serde::{Deserialize, Serialize};
use serde_tuple::*;
use std::ops::Deref;

#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Font {
    pub name: String,
    pub style: Style,
    pub size: Size,
}

pub fn font(typeface_name: impl AsRef<str>, size: scalar) -> Font {
    Font::new(typeface_name.as_ref(), Style::NORMAL, Size::new(size))
}

impl Font {
    pub fn new(name: &str, style: Style, size: Size) -> Self {
        Font {
            name: String::from(name),
            style,
            size,
        }
    }
}

// TODO: don't serialize defaults?
#[derive(Copy, Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Default, Debug)]
pub struct Style {
    pub weight: Weight,
    pub width: Width,
    pub slant: Slant,
}

impl Style {
    pub const NORMAL: Self = Style::new(Weight::NORMAL, Width::NORMAL, Slant::Upright);
    pub const BOLD: Self = Style::new(Weight::BOLD, Width::NORMAL, Slant::Upright);
    pub const ITALIC: Self = Style::new(Weight::NORMAL, Width::NORMAL, Slant::Italic);
    pub const BOLD_ITALIC: Self = Style::new(Weight::BOLD, Width::NORMAL, Slant::Italic);

    pub const fn new(weight: Weight, width: Width, slant: Slant) -> Self {
        Style {
            weight,
            width,
            slant,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Size(scalar);

// TODO: may use derive_more.
impl Deref for Size {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Size {
    pub const fn new(size: scalar) -> Self {
        Self(size)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Weight(usize);

impl Default for Weight {
    fn default() -> Self {
        Weight::NORMAL
    }
}

impl Deref for Weight {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Weight {
    pub const INVISIBLE: Self = Self(0);
    pub const THIN: Self = Self(100);
    pub const EXTRA_LIGHT: Self = Self(200);
    pub const LIGHT: Self = Self(300);
    pub const NORMAL: Self = Self(400);
    pub const MEDIUM: Self = Self(500);
    pub const SEMI_BOLD: Self = Self(600);
    pub const BOLD: Self = Self(700);
    pub const EXTRA_BOLD: Self = Self(800);
    pub const EXTRA_BLACK: Self = Self(900);
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Width(usize);

impl Default for Width {
    fn default() -> Self {
        Width::NORMAL
    }
}

impl Deref for Width {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Width {
    pub const ULTRA_CONDENSED: Self = Self(1);
    pub const EXTRA_CONDENSED: Self = Self(2);
    pub const CONDENSED: Self = Self(3);
    pub const SEMI_CONDENSED: Self = Self(4);
    pub const NORMAL: Self = Self(5);
    pub const SEMI_EXPANDED: Self = Self(6);
    pub const EXPANDED: Self = Self(7);
    pub const EXTRA_EXPANDED: Self = Self(8);
    pub const ULTRA_EXPANDED: Self = Self(9);
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Slant {
    Upright,
    Italic,
    Oblique,
}

impl Default for Slant {
    fn default() -> Self {
        Slant::Upright
    }
}
