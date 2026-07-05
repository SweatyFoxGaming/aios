//! Legacy VGA text-mode console (80x25, `0xB8000`).
//!
//! Serial output (`serial.rs`) is invisible on real hardware unless a
//! null-modem/USB-serial cable is attached to COM1 -- most machines have no
//! physical serial port at all. `0xB8000` is the standard VGA text-mode
//! buffer address that BIOS-legacy boots (like this kernel's GRUB2/
//! Multiboot2 + `boot32.asm` path) leave active unless something explicitly
//! switches video modes; the first 2GiB is already identity-mapped by
//! `boot32.asm`, so this address is valid as soon as paging is enabled --
//! no separate initialization needed. `println!`/`print!` write here in
//! addition to serial, so boot progress is visible directly on a monitor.

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

const BUFFER_ADDR: usize = 0xB8000;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const LIGHT_GRAY_ON_BLACK: u8 = 0x07;

struct VgaWriter {
    col: usize,
    row: usize,
}

impl VgaWriter {
    const fn new() -> Self {
        Self { col: 0, row: 0 }
    }

    fn buffer(&mut self) -> *mut u16 {
        BUFFER_ADDR as *mut u16
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col >= WIDTH {
                    self.newline();
                }
                let offset = self.row * WIDTH + self.col;
                let entry = (u16::from(LIGHT_GRAY_ON_BLACK) << 8) | u16::from(byte);
                unsafe {
                    self.buffer().add(offset).write_volatile(entry);
                }
                self.col += 1;
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 >= HEIGHT {
            self.scroll();
        } else {
            self.row += 1;
        }
    }

    fn scroll(&mut self) {
        unsafe {
            let buf = self.buffer();
            for row in 1..HEIGHT {
                for col in 0..WIDTH {
                    let entry = buf.add(row * WIDTH + col).read_volatile();
                    buf.add((row - 1) * WIDTH + col).write_volatile(entry);
                }
            }
            let blank = (u16::from(LIGHT_GRAY_ON_BLACK) << 8) | u16::from(b' ');
            for col in 0..WIDTH {
                buf.add((HEIGHT - 1) * WIDTH + col).write_volatile(blank);
            }
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // Only printable ASCII + newline; anything else (e.g. UTF-8
            // continuation bytes from names like "Kairòs") renders as a
            // fallback glyph rather than corrupting cell alignment.
            match byte {
                b'\n' | 0x20..=0x7e => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
        Ok(())
    }
}

lazy_static! {
    static ref WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new());
}

/// Internal print function, mirroring `serial::_print`'s signature.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    let _ = WRITER.lock().write_fmt(args);
}
