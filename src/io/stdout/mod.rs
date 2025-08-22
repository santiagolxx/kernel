//! Módulo de salida estándar (stdout) para el kernel
//!
//! Proporciona funcionalidades para escribir texto en el buffer VGA
//! de forma segura y eficiente.

use crate::io::stdout::structs::SysPrintableChar;
use heapless::Vec;

// Submódulos
pub mod colors;
pub mod structs;

/// Constantes del buffer VGA
pub const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
pub const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT * 2;

/// Escribe un carácter en una posición específica del buffer VGA
///
/// Esta función es segura siempre que el carácter esté dentro de los límites
/// de la pantalla. Los caracteres fuera de límites son ignorados silenciosamente.
///
/// # Argumentos
/// * `syschar` - El carácter a escribir con su posición y color
///
/// # Seguridad
/// Esta función usa punteros raw para acceder al buffer VGA, pero incluye
/// verificaciones de límites para prevenir escrituras fuera del buffer.
pub fn write_char_at(syschar: structs::SysPrintableChar) {
    // Verificar que el carácter esté dentro de los límites
    if !syschar.is_valid() {
        return; // Ignorar silenciosamente caracteres fuera de límites
    }

    let index = (syschar.y as usize * VGA_WIDTH + syschar.x as usize) * 2;

    // Verificación adicional del índice por seguridad
    if index + 1 < VGA_BUFFER_SIZE {
        unsafe {
            *VGA_BUFFER.add(index) = syschar.character;
            *VGA_BUFFER.add(index + 1) = syschar.color;
        }
    }
}

/// Escribe múltiples caracteres desde un buffer
///
/// Esta función toma un vector de caracteres y los escribe todos
/// al buffer VGA de forma eficiente.
///
/// # Argumentos
/// * `vec` - Vector de caracteres a escribir
///
/// # Tipo genérico
/// * `N` - Tamaño máximo del vector (determinado en tiempo de compilación)
pub fn write_buffer<const N: usize>(vec: Vec<SysPrintableChar, N>) {
    for char in vec {
        write_char_at(char);
    }
}

/// Limpia la pantalla completa
///
/// Llena toda la pantalla con espacios en blanco usando el color especificado.
pub fn clear_screen(background_color: u8) {
    use crate::io::stdout::colors::BLACK;

    let clear_char = structs::SysPrintableChar::new(b' ', background_color, 0, 0);

    for y in 0..VGA_HEIGHT {
        for x in 0..VGA_WIDTH {
            let char_at_pos =
                structs::SysPrintableChar::new(b' ', background_color, x as u8, y as u8);
            write_char_at(char_at_pos);
        }
    }
}

/// Escribe una línea de texto en una posición específica
///
/// Función de conveniencia para escribir strings simples.
///
/// # Argumentos
/// * `text` - El texto a escribir
/// * `color` - Color del texto
/// * `x` - Posición X inicial
/// * `y` - Posición Y inicial
pub fn write_string_at(text: &[u8], color: u8, x: u8, y: u8) {
    let chars = structs::SysPrintableChar::new_string(text, color, x, y);
    write_buffer(chars);
}

/// Estructura para manejar un "cursor" de escritura
///
/// Permite escribir texto secuencialmente sin tener que calcular
/// posiciones manualmente.
pub struct TextCursor {
    x: u8,
    y: u8,
    color: u8,
}

impl TextCursor {
    /// Crea un nuevo cursor en la posición especificada
    pub fn new(x: u8, y: u8, color: u8) -> Self {
        Self { x, y, color }
    }

    /// Escribe un carácter en la posición actual del cursor
    pub fn write_char(&mut self, character: u8) {
        if self.is_valid_position() {
            let syschar = structs::SysPrintableChar::new(character, self.color, self.x, self.y);
            write_char_at(syschar);
            self.advance();
        }
    }

    /// Escribe una string en la posición actual del cursor
    pub fn write_string(&mut self, text: &[u8]) {
        for &byte in text {
            match byte {
                b'\n' => self.new_line(),
                b'\r' => self.carriage_return(),
                _ => self.write_char(byte),
            }
        }
    }

    /// Avanza el cursor una posición
    fn advance(&mut self) {
        self.x += 1;
        if self.x >= VGA_WIDTH as u8 {
            self.new_line();
        }
    }

    /// Mueve el cursor al inicio de la siguiente línea
    fn new_line(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y >= VGA_HEIGHT as u8 {
            self.y = VGA_HEIGHT as u8 - 1; // Mantener en la última línea
        }
    }

    /// Mueve el cursor al inicio de la línea actual
    fn carriage_return(&mut self) {
        self.x = 0;
    }

    /// Verifica si el cursor está en una posición válida
    fn is_valid_position(&self) -> bool {
        (self.x as usize) < VGA_WIDTH && (self.y as usize) < VGA_HEIGHT
    }

    /// Obtiene la posición actual del cursor
    pub fn position(&self) -> (u8, u8) {
        (self.x, self.y)
    }

    /// Cambia el color del cursor
    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }
}
