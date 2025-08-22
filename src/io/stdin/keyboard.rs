/// Controlador del teclado PS/2
///
/// Este módulo proporciona funcionalidades básicas para interactuar con
/// el controlador de teclado PS/2, incluyendo inicialización, lectura de
/// scancodes y conversión a caracteres ASCII.

/// Puertos del controlador de teclado PS/2
mod ports {
    pub const DATA: u16 = 0x60; // Puerto de datos (lectura/escritura)
    pub const STATUS_CMD: u16 = 0x64; // Puerto de estado/comando
}

/// Bits del registro de estado
mod status_bits {
    pub const OUTPUT_BUFFER_FULL: u8 = 0x01; // Buffer de salida lleno
    pub const INPUT_BUFFER_FULL: u8 = 0x02; // Buffer de entrada lleno
}

/// Comandos básicos del teclado
mod commands {
    pub const READ_CONFIG: u8 = 0x20;
    pub const WRITE_CONFIG: u8 = 0x60;
    pub const DISABLE_KEYBOARD: u8 = 0xAD;
    pub const ENABLE_KEYBOARD: u8 = 0xAE;
}

/// Lee un byte desde un puerto de I/O de forma segura
#[inline]
unsafe fn read_port(port: u16) -> u8 {
    let result: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") result,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    result
}

/// Escribe un byte a un puerto de I/O de forma segura
#[inline]
unsafe fn write_port(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

/// Verifica si hay datos disponibles en el buffer de salida
fn is_output_ready() -> bool {
    unsafe {
        let status = read_port(ports::STATUS_CMD);
        (status & status_bits::OUTPUT_BUFFER_FULL) != 0
    }
}

/// Verifica si el buffer de entrada está listo para recibir comandos
fn is_input_ready() -> bool {
    unsafe {
        let status = read_port(ports::STATUS_CMD);
        (status & status_bits::INPUT_BUFFER_FULL) == 0
    }
}

/// Espera hasta que el input buffer esté listo
fn wait_for_input_ready() {
    let mut timeout = 0;
    while !is_input_ready() && timeout < 1000000 {
        core::hint::spin_loop();
        timeout += 1;
    }
}

/// Espera hasta que haya datos disponibles
fn wait_for_output_ready() {
    let mut timeout = 0;
    while !is_output_ready() && timeout < 1000000 {
        core::hint::spin_loop();
        timeout += 1;
    }
}

/// Lee un scancode del teclado (no bloqueante)
pub fn read_scancode() -> Option<u8> {
    if is_output_ready() {
        unsafe { Some(read_port(ports::DATA)) }
    } else {
        None
    }
}

/// Envía un comando al controlador del teclado
fn send_command(cmd: u8) {
    unsafe {
        wait_for_input_ready();
        write_port(ports::STATUS_CMD, cmd);
    }
}

/// Envía datos al teclado
fn send_data(data: u8) {
    unsafe {
        wait_for_input_ready();
        write_port(ports::DATA, data);
    }
}

/// Inicializa el teclado con configuración básica
pub fn init_keyboard() {
    unsafe {
        // Habilitar el teclado
        send_command(commands::ENABLE_KEYBOARD);

        // Leer configuración actual
        send_command(commands::READ_CONFIG);
        wait_for_output_ready();

        let config = read_port(ports::DATA);

        // Habilitar interrupciones del teclado y deshabilitar las del mouse
        let new_config = (config | 0x01) & !0x20;

        // Escribir nueva configuración
        send_command(commands::WRITE_CONFIG);
        send_data(new_config);
    }
}

/// Tabla de conversión de scancode a ASCII (layout US básico)
const SCANCODE_MAP: [Option<char>; 256] = {
    let mut map = [None; 256];

    // Números
    map[0x02] = Some('1');
    map[0x03] = Some('2');
    map[0x04] = Some('3');
    map[0x05] = Some('4');
    map[0x06] = Some('5');
    map[0x07] = Some('6');
    map[0x08] = Some('7');
    map[0x09] = Some('8');
    map[0x0A] = Some('9');
    map[0x0B] = Some('0');

    // Fila QWERTY
    map[0x10] = Some('q');
    map[0x11] = Some('w');
    map[0x12] = Some('e');
    map[0x13] = Some('r');
    map[0x14] = Some('t');
    map[0x15] = Some('y');
    map[0x16] = Some('u');
    map[0x17] = Some('i');
    map[0x18] = Some('o');
    map[0x19] = Some('p');

    // Fila ASDF
    map[0x1E] = Some('a');
    map[0x1F] = Some('s');
    map[0x20] = Some('d');
    map[0x21] = Some('f');
    map[0x22] = Some('g');
    map[0x23] = Some('h');
    map[0x24] = Some('j');
    map[0x25] = Some('k');
    map[0x26] = Some('l');

    // Fila ZXCV
    map[0x2C] = Some('z');
    map[0x2D] = Some('x');
    map[0x2E] = Some('c');
    map[0x2F] = Some('v');
    map[0x30] = Some('b');
    map[0x31] = Some('n');
    map[0x32] = Some('m');

    // Teclas especiales
    map[0x39] = Some(' '); // Espacio
    map[0x1C] = Some('\n'); // Enter

    map
};

/// Convierte un scancode a carácter ASCII
#[inline]
fn scancode_to_char(scancode: u8) -> Option<char> {
    SCANCODE_MAP.get(scancode as usize).copied().flatten()
}

/// Obtiene entrada del teclado mediante polling (no bloqueante)
///
/// Retorna Some(char) si se presionó una tecla válida,
/// None si no hay entrada o la tecla no está mapeada
pub fn poll_keyboard() -> Option<char> {
    if let Some(scancode) = read_scancode() {
        // Verificar que es una pulsación (no liberación de tecla)
        if scancode & 0x80 == 0 {
            return scancode_to_char(scancode);
        }
    }
    None
}
