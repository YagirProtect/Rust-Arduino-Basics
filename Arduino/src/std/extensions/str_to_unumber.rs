pub trait StrToNumberExt {
    fn to_u8(self) -> u8;
    fn to_u16(self) -> u16;
}

impl StrToNumberExt for &str {
    fn to_u8(self) -> u8 {
        let bytes = self.as_bytes();
        let mut i = 0;
        let mut n = 0u8;

        while i < bytes.len() {
            let b = bytes[i];
            if b >= b'0' && b <= b'9' {
                n = n * 10 + (b - b'0');
            }
            i += 1;
        }

        n
    }

    fn to_u16(self) -> u16 {
        let bytes = self.as_bytes();
        let mut i = 0;
        let mut n = 0u16;

        while i < bytes.len() {
            let b = bytes[i];
            if b >= b'0' && b <= b'9' {
                n = n * 10 + (b - b'0') as u16;
            }
            i += 1;
        }

        n
    }
}