use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Font(pub String, pub Style, pub Size);

// TODO: don't serialize defaults?
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Style(pub Weight, pub Width, pub Slant);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Size(pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Weight(pub usize);
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Width(pub usize);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Slant {
    Upright,
    Italic,
    Oblique,
}

impl Font {
    pub fn new(name: &str, style: Style, size: Size) -> Self {
        Self(String::from(name), style, size)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn style(&self) -> Style {
        self.1
    }

    pub fn size(&self) -> Size {
        self.2
    }
}

impl Style {
    pub const NORMAL: Self = Style(Weight::NORMAL, Width::NORMAL, Slant::Upright);
    pub const BOLD: Self = Style(Weight::BOLD, Width::NORMAL, Slant::Upright);
    pub const ITALIC: Self = Style(Weight::NORMAL, Width::NORMAL, Slant::Italic);
    pub const BOLD_ITALIC: Self = Style(Weight::BOLD, Width::NORMAL, Slant::Italic);

    pub fn weight(&self) -> Weight {
        self.0
    }

    pub fn width(&self) -> Width {
        self.1
    }

    pub fn slant(&self) -> Slant {
        self.2
    }
}

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

impl Default for Slant {
    fn default() -> Self {
        Slant::Upright
    }
}

impl Deref for Size {
    type Target = scalar;
    fn deref(&self) -> &scalar {
        &self.0
    }
}
