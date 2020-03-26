use crate::app::gen::enums::{OpType, TripCloseCode};
use crate::util::cursor::{ReadCursor, ReadError, WriteCursor, WriteError};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DoubleBit {
    Intermediate,
    DeterminedOff,
    DeterminedOn,
    Indeterminate,
}

impl DoubleBit {
    // the lowest two bits of this number
    pub fn from(x: u8) -> Self {
        match x & 0b0000_0011 {
            0b00 => DoubleBit::Intermediate,
            0b01 => DoubleBit::DeterminedOff,
            0b10 => DoubleBit::DeterminedOn,
            _ => DoubleBit::Indeterminate,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Control {
    pub fir: bool,
    pub fin: bool,
    pub con: bool,
    pub uns: bool,
    pub seq: u8,
}

impl Control {
    const FIR_MASK: u8 = 0b1000_0000;
    const FIN_MASK: u8 = 0b0100_0000;
    const CON_MASK: u8 = 0b0010_0000;
    const UNS_MASK: u8 = 0b0001_0000;
    const SEQ_MASK: u8 = 0b0000_1111;

    pub fn from(x: u8) -> Self {
        Self {
            fir: x & Self::FIR_MASK != 0,
            fin: x & Self::FIN_MASK != 0,
            con: x & Self::CON_MASK != 0,
            uns: x & Self::UNS_MASK != 0,
            seq: x & Self::SEQ_MASK,
        }
    }

    pub fn to_u8(self) -> u8 {
        let mut x: u8 = 0;
        if self.fir {
            x |= Self::FIR_MASK;
        }
        if self.fin {
            x |= Self::FIN_MASK;
        }
        if self.con {
            x |= Self::CON_MASK;
        }
        if self.uns {
            x |= Self::UNS_MASK;
        }
        x |= self.seq & Self::SEQ_MASK;
        x
    }

    pub fn parse(cursor: &mut ReadCursor) -> Result<Self, ReadError> {
        Ok(Self::from(cursor.read_u8()?))
    }

    pub fn write(&self, cursor: &mut WriteCursor) -> Result<(), WriteError> {
        Ok(cursor.write_u8(self.to_u8())?)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IIN {
    pub iin1: u8,
    pub iin2: u8,
}

impl IIN {
    pub fn new(iin1: u8, iin2: u8) -> Self {
        Self { iin1, iin2 }
    }

    pub fn parse(cursor: &mut ReadCursor) -> Result<Self, ReadError> {
        Ok(Self {
            iin1: cursor.read_u8()?,
            iin2: cursor.read_u8()?,
        })
    }

    pub fn write(&self, cursor: &mut WriteCursor) -> Result<(), WriteError> {
        cursor.write_u8(self.iin1)?;
        cursor.write_u8(self.iin2)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ControlCode {
    pub tcc: TripCloseCode,
    pub clear: bool,
    pub queue: bool,
    pub op_type: OpType,
}

impl ControlCode {
    const TCC_MASK: u8 = 0b1100_0000;
    const CR_MASK: u8 = 0b0010_0000;
    const QU_MASK: u8 = 0b0001_0000;
    const OP_MASK: u8 = 0b0000_1111;

    pub fn from(x: u8) -> Self {
        Self {
            tcc: TripCloseCode::from((x & Self::TCC_MASK) >> 6),
            clear: x & Self::CR_MASK != 0,
            queue: x & Self::QU_MASK != 0,
            op_type: OpType::from(x & Self::OP_MASK),
        }
    }
    pub fn as_u8(self) -> u8 {
        let mut x = 0;
        x |= self.tcc.as_u8() << 6;
        if self.clear {
            x |= Self::CR_MASK;
        }
        if self.queue {
            x |= Self::QU_MASK;
        }
        x |= self.op_type.as_u8();
        x
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_control_code_round_trip(byte: u8, cc: ControlCode) {
        assert_eq!(cc.as_u8(), byte);
        assert_eq!(ControlCode::from(byte), cc)
    }

    #[test]
    fn correctly_converts_control_code_to_and_from_u8() {
        test_control_code_round_trip(
            0b10_1_1_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: true,
                queue: true,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b10_0_1_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: false,
                queue: true,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b10_1_0_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: true,
                queue: false,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b11_0_0_0000,
            ControlCode {
                tcc: TripCloseCode::Reserved,
                clear: false,
                queue: false,
                op_type: OpType::Nul,
            },
        );
    }
}
