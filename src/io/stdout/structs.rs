use crate::io::stdout::{VGA_HEIGHT, VGA_WIDTH};

/// Representa un carácter imprimible en el sistema VGA
///
/// Esta estructura encapsula toda la información necesaria para
/// mostrar un carácter en una posición específica de la pantalla
/// con un color determinado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SysPrintableChar {
    pub character: u8,
    pub color: u8,
    pub x: u8,
    pub y: u8,
}

impl SysPrintableChar {
    /// Crea un nuevo carácter imprimible
    ///
    /// # Argumentos
    /// * `character` - El carácter ASCII a mostrar
    /// * `color` - El color del carácter (foreground + background)
    /// * `x` - Posición horizontal (0-79)
    /// * `y` - Posición vertical (0-24)
    #[inline(always)]
    pub const fn new(character: u8, color: u8, x: u8, y: u8) -> Self {
        Self {
            character,
            color,
            x,
            y,
        }
    }

    /// Crea una serie de caracteres desde un string
    ///
    /// Esta función toma un buffer de bytes y lo convierte en una serie
    /// de caracteres imprimibles, manejando automáticamente el salto de línea
    /// y el wrapping de texto.
    ///
    /// # Argumentos
    /// * `buffer` - Los bytes a convertir
    /// * `color` - Color a aplicar a todos los caracteres
    /// * `start_x` - Posición X inicial
    /// * `start_y` - Posición Y inicial
    ///
    /// # Retorna
    /// Un vector de caracteres listos para ser mostrados
    pub fn new_string(
        buffer: &[u8],
        color: u8,
        start_x: u8,
        start_y: u8,
    ) -> heapless::Vec<Self, 2000> {
        let mut chars = heapless::Vec::new();
        let mut position = TextPosition::new(start_x, start_y);

        for &byte in buffer {
            // Verificar límites antes de procesar
            if position.is_out_of_bounds() || chars.is_full() {
                break;
            }

            match byte {
                b'\n' => {
                    position.new_line(start_x);
                }
                b'\r' => {
                    position.carriage_return(start_x);
                }
                b'\t' => {
                    // Tab = 4 espacios
                    for _ in 0..4 {
                        if position.is_out_of_bounds() || chars.is_full() {
                            break;
                        }
                        let _ = chars.push(Self::new(b' ', color, position.x, position.y));
                        position.advance();
                    }
                }
                printable_char if printable_char.is_ascii_graphic() || printable_char == b' ' => {
                    // Manejar wrap automático
                    if position.x >= VGA_WIDTH as u8 {
                        position.new_line(start_x);
                        if position.is_out_of_bounds() {
                            break;
                        }
                    }

                    let _ = chars.push(Self::new(printable_char, color, position.x, position.y));
                    position.advance();
                }
                _ => {
                    // Ignorar caracteres no imprimibles
                }
            }
        }

        chars
    }

    /// Crea un iterador de caracteres desde un string (sin allocación)
    ///
    /// Esta versión es más eficiente en memoria ya que no pre-aloca
    /// todo el vector, sino que genera caracteres on-demand.
    pub fn chars_from_string(
        buffer: &[u8],
        color: u8,
        start_x: u8,
        start_y: u8,
    ) -> impl Iterator<Item = SysPrintableChar> + '_ {
        let mut position = TextPosition::new(start_x, start_y);

        buffer.iter().filter_map(move |&byte| {
            if position.is_out_of_bounds() {
                return None;
            }

            match byte {
                b'\n' => {
                    position.new_line(start_x);
                    None
                }
                b'\r' => {
                    position.carriage_return(start_x);
                    None
                }
                printable_char if printable_char.is_ascii_graphic() || printable_char == b' ' => {
                    // Wrap automático
                    if position.x >= VGA_WIDTH as u8 {
                        position.new_line(start_x);
                        if position.is_out_of_bounds() {
                            return None;
                        }
                    }

                    let result =
                        SysPrintableChar::new(printable_char, color, position.x, position.y);
                    position.advance();
                    Some(result)
                }
                _ => None, // Ignorar caracteres no imprimibles
            }
        })
    }

    /// Verifica si el carácter está dentro de los límites de la pantalla
    pub fn is_valid(&self) -> bool {
        (self.x as usize) < VGA_WIDTH && (self.y as usize) < VGA_HEIGHT
    }
}

/// Estructura auxiliar para manejar posiciones de texto
#[derive(Debug, Clone, Copy)]
struct TextPosition {
    x: u8,
    y: u8,
}

impl TextPosition {
    const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    fn advance(&mut self) {
        self.x += 1;
    }

    fn new_line(&mut self, start_x: u8) {
        self.y += 1;
        self.x = start_x;
    }

    fn carriage_return(&mut self, start_x: u8) {
        self.x = start_x;
    }

    fn is_out_of_bounds(&self) -> bool {
        self.y >= VGA_HEIGHT as u8
    }
}
