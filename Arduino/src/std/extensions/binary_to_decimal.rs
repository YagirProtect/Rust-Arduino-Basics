pub trait BcdExt {
    fn bdc_to_dec(self) -> u8;
    fn dec_to_bdc(self) -> u8;
}

impl BcdExt for u8 {
    //Binary Decoded Decimal to decimal
    fn bdc_to_dec(self) -> u8 {
        ((self >> 4) * 10) + (self & 0x0F)
    }

    fn dec_to_bdc(self) -> u8 {
        ((self / 10) << 4) | (self % 10)
    }
}