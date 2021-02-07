use crate::bus::Bus;
use anyhow::{bail, Context, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone, Debug)]
enum Opecode {
    Nop = 0x00,
    Ld8Bim8 = 0x06,
    Ld8Cim8 = 0x0E,
    Ld8Dim8 = 0x16,
    Ld8Eim8 = 0x1E,
    Ld8Him8 = 0x26,
    Ld8Lim8 = 0x2E,
    Ld8AA = 0x7F,
    Ld8AB = 0x78,
    Ld8AC = 0x79,
    Ld8AD = 0x7A,
    Ld8AE = 0x7B,
    Ld8AH = 0x7C,
    Ld8AL = 0x7D,
    Ld8AaddrHl = 0x7E,
    Ld8BB = 0x40,
    Ld8BC = 0x41,
    Ld8BD = 0x42,
    Ld8BE = 0x43,
    Ld8BH = 0x44,
    Ld8BL = 0x45,
    Ld8BaddrHl = 0x46,
    Ld8CB = 0x48,
    Ld8CC = 0x49,
    Ld8CD = 0x4A,
    Ld8CE = 0x4B,
    Ld8CH = 0x4C,
    Ld8CL = 0x4D,
    Ld8CaddrHl = 0x4E,
    Ld8DB = 0x50,
    Ld8DC = 0x51,
    Ld8DD = 0x52,
    Ld8DE = 0x53,
    Ld8DH = 0x54,
    Ld8DL = 0x55,
    Ld8DaddrHl = 0x56,
    Ld8EB = 0x58,
    Ld8EC = 0x59,
    Ld8ED = 0x5A,
    Ld8EE = 0x5B,
    Ld8EH = 0x5C,
    Ld8EL = 0x5D,
    Ld8EaddrHl = 0x5E,
    Ld8HB = 0x60,
    Ld8HC = 0x61,
    Ld8HD = 0x62,
    Ld8HE = 0x63,
    Ld8HH = 0x64,
    Ld8HL = 0x65,
    Ld8HaddrHl = 0x66,
    Ld8LB = 0x68,
    Ld8LC = 0x69,
    Ld8LD = 0x6A,
    Ld8LE = 0x6B,
    Ld8LH = 0x6C,
    Ld8LL = 0x6D,
    Ld8LaddrHl = 0x6E,
    Ld8addrHlB = 0x70,
    Ld8addrHlC = 0x71,
    Ld8addrHlD = 0x72,
    Ld8addrHlE = 0x73,
    Ld8addrHlH = 0x74,
    Ld8addrHlL = 0x75,
    Ld8addrHlIm8 = 0x36,
    Ld8AaddrBc = 0x0A,
    Ld8AaddrDe = 0x1A,
    Ld8Aaddr16 = 0xFA,
    Ld8Aim8 = 0x3E,
    Ld8BA = 0x47,
    Ld8CA = 0x4F,
    Ld8DA = 0x57,
    Ld8EA = 0x5F,
    Ld8HA = 0x67,
    Ld8LA = 0x6F,
    Ld8addrBcA = 0x02,
    Ld8addrDeA = 0x12,
    Ld8addrHlA = 0x77,
    Ld8addr16A = 0xEA,
    Ld8AaddrIndexC = 0xF2,
    Ld8addrIndexCA = 0xE2,
    Ldd8AaddrHl = 0x3A,
    Ldd8addrHlA = 0x32,
    Ldi8AaddrHl = 0x2A,
    LdiaddrHlA = 0x22,
    LdhaddrIndex8A = 0xE0,
    LdhAaddrIndex8 = 0xF0,
    Ld16Bcim16 = 0x01,
    Ld16Deim16 = 0x11,
    Ld16Hlim16 = 0x21,
    Ld16Spim16 = 0x31,
    Ld16SpHl = 0xF9,
    Ldhl16Spim8 = 0xF8,
    Ldh16addr16Sp = 0x08,
    Push16Af = 0xF5,
    Push16Bc = 0xC5,
    Push16De = 0xD5,
    Push16Hl = 0xE5,
    Pop16Af = 0xF1,
    Pop16Bc = 0xC1,
    Pop16De = 0xD1,
    Pop16Hl = 0xE1,
    Add8AA = 0x87,
    Add8AB = 0x80,
    Add8AC = 0x81,
    Add8AD = 0x82,
    Add8AE = 0x83,
    Add8AH = 0x84,
    Add8AL = 0x85,
    Add8AaddrHl = 0x86,
    Add8Aim8 = 0xC6,
    Adc8AA = 0x8F,
    Adc8AB = 0x88,
    Adc8AC = 0x89,
    Adc8AD = 0x8A,
    Adc8AE = 0x8B,
    Adc8AH = 0x8C,
    Adc8AL = 0x8D,
    Adc8AaddrHl = 0x8E,
    Adc8Aim8 = 0xCE,
    Sub8A = 0x97,
    Sub8B = 0x90,
    Sub8C = 0x91,
    Sub8D = 0x92,
    Sub8E = 0x93,
    Sub8H = 0x94,
    Sub8L = 0x95,
    Sub8addrHl = 0x96,
    Sub8im8 = 0xD6,
    Sbc8AA = 0x9F,
    Sbc8AB = 0x98,
    Sbc8AC = 0x99,
    Sbc8AD = 0x9A,
    Sbc8AE = 0x9B,
    Sbc8AH = 0x9C,
    Sbc8AL = 0x9D,
    Sbc8AaddrHl = 0x9E,
    And8A = 0xA7,
    And8B = 0xA0,
    And8C = 0xA1,
    And8D = 0xA2,
    And8E = 0xA3,
    And8H = 0xA4,
    And8L = 0xA5,
    And8addrHl = 0xA6,
    And8im8 = 0xE6,
    Or8A = 0xB7,
    Or8B = 0xB0,
    Or8C = 0xB1,
    Or8D = 0xB2,
    Or8E = 0xB3,
    Or8H = 0xB4,
    Or8L = 0xB5,
    Or8addrHl = 0xB6,
    Or8im8 = 0xF6,
    Xor8A = 0xAF,
    Xor8B = 0xA8,
    Xor8C = 0xA9,
    Xor8D = 0xAA,
    Xor8E = 0xAB,
    Xor8H = 0xAC,
    Xor8L = 0xAD,
    Xor8addrHl = 0xAE,
    Xor8im8 = 0xEE,
    Cp8A = 0xBF,
    Cp8B = 0xB8,
    Cp8C = 0xB9,
    Cp8D = 0xBA,
    Cp8E = 0xBB,
    Cp8H = 0xBC,
    Cp8L = 0xBD,
    Cp8addrHl = 0xBE,
    Cp8im8 = 0xFE,
    Inc8A = 0x3C,
    Inc8B = 0x04,
    Inc8C = 0x0C,
    Inc8D = 0x14,
    Inc8E = 0x1C,
    Inc8H = 0x24,
    Inc8L = 0x2C,
    Inc8addrHl = 0x34,
    Dec8A = 0x3D,
    Dec8B = 0x05,
    Dec8C = 0x0D,
    Dec8D = 0x15,
    Dec8E = 0x1D,
    Dec8H = 0x25,
    Dec8L = 0x2D,
    Dec8addrHl = 0x35,
    Add16HlBc = 0x09,
    Add16HlDe = 0x19,
    Add16HlHl = 0x29,
    Add16HlSp = 0x39,
    Add16Spim8 = 0xE8,
    Inc16Bc = 0x03,
    Inc16De = 0x13,
    Inc16Hl = 0x23,
    Inc16Sp = 0x33,
    Dec16Bc = 0x0B,
    Dec16De = 0x1B,
    Dec16Hl = 0x2B,
    Dec16Sp = 0x3B,
    Prefix = 0xCB,
    Daa = 0x27,
    Cpl = 0x2F,
    Ccf = 0x3F,
    Scf = 0x37,
    Halt = 0x76,
    Stop = 0x10,
    Di = 0xF3,
    Ei = 0xFB,
    Rlca = 0x07,
    Rla = 0x17,
    Rrca = 0x0F,
    Rra = 0x1F,
    Jpaddr16 = 0xC3,
    JpNzaddr16 = 0xC2,
    JpZaddr16 = 0xCA,
    JpNcaddr16 = 0xD2,
    JpCaddr16 = 0xDA,
    JpaddrHl = 0xE9,
    JraddrIndex8 = 0x18,
    JrNzaddrIndex8 = 0x20,
    JrZaddrIndex8 = 0x28,
    JrNcaddrIndex8 = 0x30,
    JrCaddrIndex8 = 0x38,
    Calladdr16 = 0xCD,
    CallNzaddr16 = 0xC4,
    CallZaddr16 = 0xCC,
    CallNcaddr16 = 0xD4,
    CallCaddr16 = 0xDC,
    Rst00h = 0xC7,
    Rst08h = 0xCF,
    Rst10h = 0xD7,
    Rst18h = 0xDF,
    Rst20h = 0xE7,
    Rst28h = 0xEF,
    Rst30h = 0xF7,
    Rst38h = 0xFF,
    Ret = 0xC9,
    RetNz = 0xC0,
    RetZ = 0xC8,
    RetNc = 0xD0,
    RetC = 0xD8,
    Reti = 0xD9,
}

#[derive(FromPrimitive, Copy, Clone, Debug)]
enum PrefixedOpecode {
    SwapA = 0x37,
    SwapB = 0x30,
    SwapC = 0x31,
    SwapD = 0x32,
    SwapE = 0x33,
    SwapH = 0x34,
    SwapL = 0x35,
    SwapaddrHl = 0x36,
    RlcA = 0x07,
    RlcB = 0x00,
    RlcC = 0x01,
    RlcD = 0x02,
    RlcE = 0x03,
    RlcH = 0x04,
    RlcL = 0x05,
    RlcaddrHl = 0x06,
    RladdrA = 0x17,
    RladdrB = 0x10,
    RladdrC = 0x11,
    RladdrD = 0x12,
    RladdrE = 0x13,
    RladdrH = 0x14,
    RladdrL = 0x15,
    RladdrHl = 0x16,
    RrcA = 0x0F,
    RrcB = 0x08,
    RrcC = 0x09,
    RrcD = 0x0A,
    RrcE = 0x0B,
    RrcH = 0x0C,
    RrcL = 0x0D,
    RrcaddrHl = 0x0E,
    RrA = 0x1F,
    RrB = 0x18,
    RrC = 0x19,
    RrD = 0x1A,
    RrE = 0x1B,
    RrH = 0x1C,
    RrL = 0x1D,
    RraddrHl = 0x1E,
    SlaA = 0x27,
    SlaB = 0x20,
    SlaC = 0x21,
    SlaD = 0x22,
    SlaE = 0x23,
    SlaH = 0x24,
    SlaL = 0x25,
    SlaaddrHl = 0x26,
    SraA = 0x2F,
    SraB = 0x28,
    SraC = 0x29,
    SraD = 0x2A,
    SraE = 0x2B,
    SraH = 0x2C,
    SraL = 0x2D,
    SraaddrHl = 0x2E,
    SrlA = 0x3F,
    SrlB = 0x38,
    SrlC = 0x39,
    SrlD = 0x3A,
    SrlE = 0x3B,
    SrlH = 0x3C,
    SrlL = 0x3D,
    SrladdrHl = 0x3E,
    BitA = 0x47,
    BitB = 0x40,
    BitC = 0x41,
    BitD = 0x42,
    BitE = 0x43,
    BitH = 0x44,
    BitL = 0x45,
    BitaddrHl = 0x46,
    Setim8A = 0xC7,
    Setim8B = 0xC0,
    Setim8C = 0xC1,
    Setim8D = 0xC2,
    Setim8E = 0xC3,
    Setim8H = 0xC4,
    Setim8L = 0xC5,
    Setim8addrHl = 0xC6,
    Resim8A = 0x87,
    Resim8B = 0x80,
    Resim8C = 0x81,
    Resim8D = 0x82,
    Resim8E = 0x83,
    Resim8H = 0x84,
    Resim8L = 0x85,
    Resim8addrHl = 0x86,
}

pub struct Cpu {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,

    stalls: u8,

    bus: Bus,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu {
            af: Default::default(),
            bc: Default::default(),
            de: Default::default(),
            hl: Default::default(),
            sp: Default::default(),
            pc: Default::default(),
            stalls: Default::default(),
            bus,
        }
    }

    pub fn reset(&mut self) {
        self.af = 0;
        self.bc = 0;
        self.de = 0;
        self.hl = 0;
        self.sp = 0;
        self.pc = 0x0100;
        self.stalls = 0;
    }

    pub fn tick(&mut self) -> Result<()> {
        // TODO interrupt

        if self.stalls > 0 {
            self.stalls -= 1;

            return Ok(());
        }

        let byte = self.bus.read(self.pc)?;

        let opecode =
            Opecode::from_u8(byte).with_context(|| format!("unknown opecode {:#X}", byte))?;

        println!("PC: {:#X}, DATA: {:#X}, OPE: {:?}", self.pc, byte, opecode);

        self.pc += 1;

        self.do_mnemonic(opecode)?;

        self.bus.tick()?;

        Ok(())
    }

    pub fn a(&self) -> u8 {
        ((self.af & 0xF0) >> 8) as u8
    }

    pub fn f(&self) -> u8 {
        (self.af & 0x0F) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.bc & 0xF0) >> 8) as u8
    }

    pub fn c(&self) -> u8 {
        (self.bc & 0x0F) as u8
    }

    pub fn d(&self) -> u8 {
        ((self.de & 0xF0) >> 8) as u8
    }

    pub fn e(&self) -> u8 {
        (self.de & 0x0F) as u8
    }

    pub fn h(&self) -> u8 {
        ((self.hl & 0xF0) >> 8) as u8
    }

    pub fn l(&self) -> u8 {
        (self.hl & 0x0F) as u8
    }

    fn set_a(&mut self, val: u8) {
        self.af &= 0x0F;
        self.af |= ((val as u16) << 8) as u16;
    }

    fn set_f(&mut self, val: u8) {
        self.af &= 0xF0;
        self.af |= val as u16;
    }

    fn set_b(&mut self, val: u8) {
        self.bc &= 0x0F;
        self.bc |= ((val as u16) << 8) as u16;
    }

    fn set_c(&mut self, val: u8) {
        self.bc &= 0xF0;
        self.bc |= val as u16;
    }

    fn set_d(&mut self, val: u8) {
        self.de &= 0x0F;
        self.de |= ((val as u16) << 8) as u16;
    }

    fn set_e(&mut self, val: u8) {
        self.de &= 0xF0;
        self.de |= val as u16;
    }

    fn set_h(&mut self, val: u8) {
        self.hl &= 0x0F;
        self.hl |= ((val as u16) << 8) as u16;
    }

    fn set_l(&mut self, val: u8) {
        self.hl &= 0xF0;
        self.hl |= val as u16;
    }

    fn r8(&self, index: u8) -> Result<u8> {
        match index {
            0 => Ok(self.b()),
            1 => Ok(self.c()),
            2 => Ok(self.d()),
            3 => Ok(self.e()),
            4 => Ok(self.h()),
            5 => Ok(self.l()),
            6 => self.bus.read(self.hl),
            7 => Ok(self.a()),
            _ => bail!("unknown r8 {}", index),
        }
    }

    fn set_r8(&mut self, index: u8, val: u8) -> Result<()> {
        match index {
            0 => {
                self.set_b(val);
                Ok(())
            }
            1 => {
                self.set_c(val);
                Ok(())
            }
            2 => {
                self.set_d(val);
                Ok(())
            }
            3 => {
                self.set_e(val);
                Ok(())
            }
            4 => {
                self.set_h(val);
                Ok(())
            }
            5 => {
                self.set_l(val);
                Ok(())
            }
            6 => self.bus.write(self.hl, val),
            7 => {
                self.set_a(val);
                Ok(())
            }
            _ => bail!("unknown r8 {}", index),
        }
    }

    pub fn flag_z(&self) -> bool {
        (self.af >> 8) & 0b10000000 > 0
    }

    pub fn flag_n(&self) -> bool {
        (self.af >> 8) & 0b01000000 > 0
    }

    pub fn flag_h(&self) -> bool {
        (self.af >> 8) & 0b00100000 > 0
    }

    pub fn flag_c(&self) -> bool {
        (self.af >> 8) & 0b00010000 > 0
    }

    pub fn flag_nz(&self) -> bool {
        (self.af >> 8) & 0b00100000 == 0
    }

    pub fn flag_nc(&self) -> bool {
        (self.af >> 8) & 0b00010000 == 0
    }

    pub fn set_flag_z(&mut self, val: bool) {
        self.af |= ((val as u16) << 8) << 7;
    }

    pub fn set_flag_n(&mut self, val: bool) {
        self.af |= ((val as u16) << 8) << 6;
    }

    pub fn set_flag_h(&mut self, val: bool) {
        self.af |= ((val as u16) << 8) << 5;
    }

    pub fn set_flag_c(&mut self, val: bool) {
        self.af |= ((val as u16) << 8) << 4;
    }

    fn do_mnemonic(&mut self, opecode: Opecode) -> Result<()> {
        let r8_index_right = opecode as u8 & 0b00000111;
        let r8_index_left = (opecode as u8 & 0b00111000) >> 3;

        match &opecode {
            Opecode::Nop => self.nop(),
            Opecode::Ld8Aim8
            | Opecode::Ld8Bim8
            | Opecode::Ld8Cim8
            | Opecode::Ld8Dim8
            | Opecode::Ld8Eim8
            | Opecode::Ld8Him8
            | Opecode::Ld8Lim8
            | Opecode::Ld8addrHlIm8 => self.load_8_r_im8(r8_index_left),
            Opecode::Ld8AA
            | Opecode::Ld8AB
            | Opecode::Ld8AC
            | Opecode::Ld8AD
            | Opecode::Ld8AE
            | Opecode::Ld8AH
            | Opecode::Ld8AL
            | Opecode::Ld8AaddrHl
            | Opecode::Ld8BB
            | Opecode::Ld8BC
            | Opecode::Ld8BD
            | Opecode::Ld8BE
            | Opecode::Ld8BH
            | Opecode::Ld8BL
            | Opecode::Ld8BaddrHl
            | Opecode::Ld8CB
            | Opecode::Ld8CC
            | Opecode::Ld8CD
            | Opecode::Ld8CE
            | Opecode::Ld8CH
            | Opecode::Ld8CL
            | Opecode::Ld8CaddrHl
            | Opecode::Ld8DB
            | Opecode::Ld8DC
            | Opecode::Ld8DD
            | Opecode::Ld8DE
            | Opecode::Ld8DH
            | Opecode::Ld8DL
            | Opecode::Ld8DaddrHl
            | Opecode::Ld8EB
            | Opecode::Ld8EC
            | Opecode::Ld8ED
            | Opecode::Ld8EE
            | Opecode::Ld8EH
            | Opecode::Ld8EL
            | Opecode::Ld8EaddrHl
            | Opecode::Ld8HB
            | Opecode::Ld8HC
            | Opecode::Ld8HD
            | Opecode::Ld8HE
            | Opecode::Ld8HH
            | Opecode::Ld8HL
            | Opecode::Ld8HaddrHl
            | Opecode::Ld8LB
            | Opecode::Ld8LC
            | Opecode::Ld8LD
            | Opecode::Ld8LE
            | Opecode::Ld8LH
            | Opecode::Ld8LL
            | Opecode::Ld8LaddrHl
            | Opecode::Ld8addrHlB
            | Opecode::Ld8addrHlC
            | Opecode::Ld8addrHlD
            | Opecode::Ld8addrHlE
            | Opecode::Ld8addrHlH
            | Opecode::Ld8addrHlL => self.load_8_r_r(r8_index_left, r8_index_right),
            Opecode::Inc8A
            | Opecode::Inc8B
            | Opecode::Inc8C
            | Opecode::Inc8D
            | Opecode::Inc8E
            | Opecode::Inc8H
            | Opecode::Inc8L
            | Opecode::Inc8addrHl => self.inc_8_r(r8_index_left),
            Opecode::Jpaddr16 => self.jp_16(),
            _ => bail!("unimplemented opecode {:?}", opecode),
        }
    }

    pub fn nop(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_8_r_im8(&mut self, index: u8) -> Result<()> {
        let val = self.bus.read(self.pc)?;

        self.pc += 1;

        self.set_r8(index, val)?;

        Ok(())
    }

    pub fn load_8_r_r(&mut self, left: u8, right: u8) -> Result<()> {
        let val = self.r8(right)?;
        self.set_r8(left, val)?;

        Ok(())
    }

    pub fn inc_8_r(&mut self, index: u8) -> Result<()> {
        let val = self.r8(index)?;
        let (res, c) = val.overflowing_add(1);

        self.set_r8(index, res)?;

        self.set_flag_z(res == 0);
        self.set_flag_n(false);
        self.set_flag_h((val & 0x08) + 1 > 0x08);
        self.set_flag_c(c);

        Ok(())
    }

    pub fn jp_16(&mut self) -> Result<()> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = addr;

        Ok(())
    }

    // TODO instructions
}
