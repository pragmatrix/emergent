use crate::config::{AnsiColors, BrightColors, Colors, NormalColors};
use crate::term::color::{List, Rgb};

pub struct Primary {
    pub background: Rgb,
    pub foreground: Rgb,
}

pub struct Scheme {
    pub primary: Primary,
    pub normal: AnsiColors,
    pub bright: AnsiColors,
}

pub mod light {
    use super::{ac, rgb, Primary, Scheme};

    pub const PENCIL: Scheme = Scheme {
        primary: Primary {
            background: rgb(0x424242),
            foreground: rgb(0xf1f1f1),
        },
        normal: ac([
            0x212121, 0xc30771, 0x10a778, 0xa89c14, 0x008ec4, 0x523c79, 0x20a5ba, 0xe0e0e0,
        ]),
        bright: ac([
            0x212121, 0xfb007a, 0x5fd7af, 0xf3e430, 0x20bbfc, 0x6855de, 0x4fb8cc, 0xf1f1f1,
        ]),
    };

    pub const SEABIRD: Scheme = Scheme {
        primary: Primary {
            background: rgb(0xffffff),
            foreground: rgb(0x61707a),
        },
        normal: ac([
            0x0b141a, 0xff4053, 0x11ab00, 0xbf8c00, 0x0099ff, 0x9854ff, 0x00a5ab, 0xffffff,
        ]),
        bright: ac([
            0x0b141a, 0xff4053, 0x11ab00, 0xbf8c00, 0x0099ff, 0x9854ff, 0x00a5ab, 0xffffff,
        ]),
    };

    pub const SOLARIZED: Scheme = Scheme {
        primary: Primary {
            background: rgb(0xfdf6e3),
            foreground: rgb(0x657b83),
        },
        normal: ac([
            0x073642, 0xdc322f, 0x859900, 0xb58900, 0x268bd2, 0xd33682, 0x2aa198, 0xeee8d5,
        ]),
        bright: ac([
            0x002b36, 0xcb4b16, 0x586e75, 0x657b83, 0x839496, 0x6c71c4, 0x93a1a1, 0xfdf6e3,
        ]),
    };

    // https://github.com/NLKNguyen/papercolor-theme/blob/master/colors/PaperColor.vim
    pub const PAPER: Scheme = Scheme {
        primary: Primary {
            background: rgb(0xffffff),
            foreground: rgb(0x000000),
        },
        normal: ac([
            0xeeeeee, 0xaf0000, 0x008700, 0x5f8700, 0x0087af, 0x878787, 0x005f87, 0x444444,
        ]),
        bright: ac([
            0xbcbcbc, 0xd70000, 0xd70087, 0x8700af, 0xd75f00, 0xd75f00, 0x005faf, 0x005f87,
        ]),
    };
}

const fn ac(colors: [u32; 8]) -> AnsiColors {
    AnsiColors {
        black: rgb(colors[0]),
        red: rgb(colors[1]),
        green: rgb(colors[2]),
        yellow: rgb(colors[3]),
        blue: rgb(colors[4]),
        magenta: rgb(colors[5]),
        cyan: rgb(colors[6]),
        white: rgb(colors[7]),
    }
}

const fn rgb(x: u32) -> Rgb {
    Rgb {
        r: (x >> 16) as u8,
        g: (x >> 8) as u8,
        b: x as u8,
    }
}

impl From<Scheme> for List {
    fn from(s: Scheme) -> Self {
        let mut colors = Colors::default();
        colors.primary.foreground = s.primary.foreground;
        colors.primary.background = s.primary.background;
        colors.normal = NormalColors(s.normal);
        colors.bright = BrightColors(s.bright);
        Self::from(&colors)
    }
}
