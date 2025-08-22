#![no_std]
#![no_main]

mod io;

use crate::io::stdout::{VGA_WIDTH, colors::LIGHT_GRAY, structs::SysPrintableChar};
use core::panic::PanicInfo;

/// Maneja los panics del kernel de forma segura y con información clara
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::io::stdout::colors::RED;
    use crate::io::stdout::{self, structs::SysPrintableChar};
    use core::fmt::Write;
    use heapless::Vec;
    use itoa::Buffer;

    // Buffer para el mensaje de panic
    let mut buffer: Vec<SysPrintableChar, 512> = Vec::new();

    // Título del panic
    let panic_title = SysPrintableChar::new_string(b"[KERNEL PANIC]", RED, 1, 0);
    let _ = buffer.extend(panic_title.iter().cloned());

    // Mensaje del panic
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        let msg_chars = SysPrintableChar::new_string(message.as_bytes(), RED, 1, 1);
        let _ = buffer.extend(msg_chars.iter().cloned());
    } else {
        let generic_msg = SysPrintableChar::new_string(b"Unknown panic occurred", RED, 1, 1);
        let _ = buffer.extend(generic_msg.iter().cloned());
    }

    // Información de ubicación si está disponible
    if let Some(location) = info.location() {
        let mut line_info: heapless::String<64> = heapless::String::new();
        let _ = write!(line_info, "at {}:{}", location.file(), location.line());

        let location_chars = SysPrintableChar::new_string(line_info.as_bytes(), RED, 1, 2);
        let _ = buffer.extend(location_chars.iter().cloned());
    }

    // Escribir todo al buffer VGA
    stdout::write_buffer(buffer);

    // Loop infinito para detener el kernel
    loop {
        core::hint::spin_loop();
    }
}

/// Estructura para manejar el estado del cursor
#[derive(Debug, Clone, Copy)]
struct CursorPosition {
    x: usize,
    y: usize,
}

impl CursorPosition {
    const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn move_to_next_line(&mut self) {
        self.y += 1;
        self.x = 0;

        // Mantener el cursor dentro de los límites de la pantalla
        if self.y >= 25 {
            self.y = 24;
        }
    }

    fn advance_cursor(&mut self) {
        self.x += 1;

        // Wrap a la siguiente línea si es necesario
        if self.x >= VGA_WIDTH {
            self.move_to_next_line();
        }
    }

    fn is_within_bounds(&self) -> bool {
        self.x < VGA_WIDTH && self.y < 25
    }
}

/// Maneja la entrada de caracteres del teclado
fn handle_character_input(character: char, cursor: &mut CursorPosition) {
    match character {
        '\n' => {
            cursor.move_to_next_line();
        }

        c if c.is_ascii() && c.is_control() == false => {
            // Solo caracteres ASCII normales (no de control)
            if cursor.is_within_bounds() {
                let display_char =
                    SysPrintableChar::new(c as u8, LIGHT_GRAY, cursor.x as u8, cursor.y as u8);
                io::stdout::write_char_at(display_char);
                cursor.advance_cursor();
            }
        }
        _ => {
            // Ignorar caracteres especiales o no ASCII
        }
    }
}

/// Punto de entrada principal del kernel
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Inicializar el teclado
    io::stdin::keyboard::init_keyboard();

    // Configurar la pantalla inicial
    let mut cursor = CursorPosition::new(1, 1);

    // Loop principal del kernel
    loop {
        // Polling del teclado
        if let Some(character) = io::stdin::keyboard::poll_keyboard() {
            handle_character_input(character, &mut cursor);
        }

        // Pequeña pausa para no sobrecargar la CPU
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }
}
