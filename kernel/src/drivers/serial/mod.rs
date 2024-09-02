pub mod uart;

pub trait ConsoleDevice {
    fn init(&self);
    fn putc(&self, ch: u8);

    fn try_getc(&self) -> Option<u8>;

    fn getc(&self) -> u8 {
        loop {
            if let Some(ch) = self.try_getc() {
                return ch;
            }
        }
    }

    fn puts(&self, s: &str) {
        for ch in s.as_bytes() {
            self.putc(*ch);
        }
    }

    fn gets(&self, buf: &mut [u8]) {
        let mut i = 0;
        loop {
            let ch = self.getc();
            if ch == b'\n' || i >= buf.len() {
                break;
            }
            buf[i] = ch;
            i += 1;
        }
    }
}
