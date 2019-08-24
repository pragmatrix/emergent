use crate::term::color::Rgb;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub primary: PrimaryColors,
    pub cursor: CursorColors,
    pub selection: SelectionColors,
    pub(crate) normal: NormalColors,
    pub(crate) bright: BrightColors,
    pub dim: Option<AnsiColors>,
    pub indexed_colors: Vec<IndexedColor>,
}

impl Colors {
    pub fn normal(&self) -> &AnsiColors {
        &self.normal.0
    }

    pub fn bright(&self) -> &AnsiColors {
        &self.bright.0
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct IndexedColor {
    pub index: u8,
    pub color: Rgb,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct CursorColors {
    pub text: Option<Rgb>,
    pub cursor: Option<Rgb>,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct SelectionColors {
    pub text: Option<Rgb>,
    pub background: Option<Rgb>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrimaryColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub bright_foreground: Option<Rgb>,
    pub dim_foreground: Option<Rgb>,
}

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            background: default_background(),
            foreground: default_foreground(),
            bright_foreground: Default::default(),
            dim_foreground: Default::default(),
        }
    }
}

fn default_background() -> Rgb {
    Rgb { r: 0, g: 0, b: 0 }
}

fn default_foreground() -> Rgb {
    Rgb {
        r: 0xea,
        g: 0xea,
        b: 0xea,
    }
}

/// The 8-colors sections of config
#[derive(Debug, PartialEq, Eq)]
pub struct AnsiColors {
    pub black: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub white: Rgb,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NormalColors(pub(crate) AnsiColors);

impl Default for NormalColors {
    fn default() -> Self {
        NormalColors(AnsiColors {
            black: Rgb {
                r: 0x00,
                g: 0x00,
                b: 0x00,
            },
            red: Rgb {
                r: 0xd5,
                g: 0x4e,
                b: 0x53,
            },
            green: Rgb {
                r: 0xb9,
                g: 0xca,
                b: 0x4a,
            },
            yellow: Rgb {
                r: 0xe6,
                g: 0xc5,
                b: 0x47,
            },
            blue: Rgb {
                r: 0x7a,
                g: 0xa6,
                b: 0xda,
            },
            magenta: Rgb {
                r: 0xc3,
                g: 0x97,
                b: 0xd8,
            },
            cyan: Rgb {
                r: 0x70,
                g: 0xc0,
                b: 0xba,
            },
            white: Rgb {
                r: 0xea,
                g: 0xea,
                b: 0xea,
            },
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BrightColors(pub(crate) AnsiColors);

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors(AnsiColors {
            black: Rgb {
                r: 0x66,
                g: 0x66,
                b: 0x66,
            },
            red: Rgb {
                r: 0xff,
                g: 0x33,
                b: 0x34,
            },
            green: Rgb {
                r: 0x9e,
                g: 0xc4,
                b: 0x00,
            },
            yellow: Rgb {
                r: 0xe7,
                g: 0xc5,
                b: 0x47,
            },
            blue: Rgb {
                r: 0x7a,
                g: 0xa6,
                b: 0xda,
            },
            magenta: Rgb {
                r: 0xb7,
                g: 0x7e,
                b: 0xe0,
            },
            cyan: Rgb {
                r: 0x54,
                g: 0xce,
                b: 0xd6,
            },
            white: Rgb {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            },
        })
    }
}
