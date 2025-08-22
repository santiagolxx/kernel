//! Definiciones de colores para el modo texto VGA

/// Colores básicos VGA (4 bits cada uno)
pub const BLACK: u8 = 0x0;
pub const BLUE: u8 = 0x1;
pub const GREEN: u8 = 0x2;
pub const CYAN: u8 = 0x3;
pub const RED: u8 = 0x4;
pub const MAGENTA: u8 = 0x5;
pub const BROWN: u8 = 0x6;
pub const LIGHT_GRAY: u8 = 0x7;
pub const DARK_GRAY: u8 = 0x8;
pub const LIGHT_BLUE: u8 = 0x9;
pub const LIGHT_GREEN: u8 = 0xa;
pub const LIGHT_CYAN: u8 = 0xb;
pub const LIGHT_RED: u8 = 0xc;
pub const LIGHT_MAGENTA: u8 = 0xd;
pub const YELLOW: u8 = 0xe;
pub const WHITE: u8 = 0xf;

/// Crea un byte de color combinando foreground y background
pub const fn make_color(foreground: u8, background: u8) -> u8 {
    (foreground & 0x0F) | ((background & 0x0F) << 4)
}

/// Extrae el color de foreground de forma segura
pub const fn get_foreground(color_byte: u8) -> u8 {
    color_byte & 0x0F
}

/// Extrae el color de background de forma segura
pub const fn get_background(color_byte: u8) -> u8 {
    (color_byte >> 4) & 0x0F
}

/// Verifica si un color es válido (0x0 - 0xF)
pub const fn is_valid_color(color: u8) -> bool {
    color <= 0xF
}

/// Crea un color con validación automática
pub const fn make_color_safe(foreground: u8, background: u8) -> u8 {
    let safe_fg = if is_valid_color(foreground) {
        foreground
    } else {
        LIGHT_GRAY
    };
    let safe_bg = if is_valid_color(background) {
        background
    } else {
        BLACK
    };
    make_color(safe_fg, safe_bg)
}
