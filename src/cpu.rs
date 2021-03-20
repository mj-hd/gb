use crate::bus::Bus;
use crate::debugger::Debugger;
use anyhow::{bail, Result};
use bitfield::bitfield;
use bitmatch::bitmatch;
use std::ops::{BitAnd, Shr};

bitfield! {
    #[derive(Default)]
    struct F(u8);
    impl Debug;
    c, set_c: 4;
    h, set_h: 5;
    n, set_n: 6;
    z, set_z: 7;
}

pub struct Cpu {
    a: u8,
    f: F,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,

    stalls: u8,

    debugger: Debugger,

    pub bus: Bus,
}

impl Cpu {
    pub fn new(bus: Bus, debugger: Debugger) -> Self {
        Cpu {
            a: Default::default(),
            f: F::default(),
            bc: Default::default(),
            de: Default::default(),
            hl: Default::default(),
            sp: Default::default(),
            pc: Default::default(),
            stalls: Default::default(),
            debugger,
            bus,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.a = 0x01;
        self.f = F(0xB0);
        self.bc = 0x0013;
        self.de = 0x00D8;
        self.hl = 0x014D;
        self.sp = 0xFFFE;
        self.pc = 0x0100;
        self.stalls = 0;

        self.bus.write(0xFF05, 0x00)?;
        self.bus.write(0xFF06, 0x00)?;
        self.bus.write(0xFF07, 0x00)?;
        self.bus.write(0xFF10, 0x80)?;
        self.bus.write(0xFF11, 0xBF)?;
        self.bus.write(0xFF12, 0xF3)?;
        self.bus.write(0xFF14, 0xF3)?;
        self.bus.write(0xFF16, 0x3F)?;
        self.bus.write(0xFF17, 0x00)?;
        self.bus.write(0xFF19, 0xBF)?;
        self.bus.write(0xFF1A, 0x7F)?;
        self.bus.write(0xFF1B, 0xFF)?;
        self.bus.write(0xFF1C, 0x9F)?;
        self.bus.write(0xFF1E, 0xBF)?;
        self.bus.write(0xFF20, 0xFF)?;
        self.bus.write(0xFF21, 0x00)?;
        self.bus.write(0xFF22, 0x00)?;
        self.bus.write(0xFF23, 0xBF)?;
        self.bus.write(0xFF24, 0x77)?;
        self.bus.write(0xFF25, 0xF3)?;
        self.bus.write(0xFF26, 0xF1)?;
        self.bus.write(0xFF40, 0x91)?;
        self.bus.write(0xFF42, 0x00)?;
        self.bus.write(0xFF43, 0x00)?;
        self.bus.write(0xFF45, 0x00)?;
        self.bus.write(0xFF47, 0xFC)?;
        self.bus.write(0xFF48, 0xFF)?;
        self.bus.write(0xFF49, 0xFF)?;
        self.bus.write(0xFF4A, 0x00)?;
        self.bus.write(0xFF4B, 0x00)?;
        self.bus.write(0xFFFF, 0x00)?;

        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        // TODO interrupt

        if self.stalls > 0 {
            self.stalls -= 1;

            return Ok(());
        }

        let opecode = self.bus.read(self.pc)?;

        let debug = self.debugger.step_run || self.debugger.breakpoints.contains(&self.pc);

        if debug {
            println!(
                "PC: {:#04X}, OPECODE: {:#02X}, A: {:#02X}, BC: {:#04X}, DE: {:#04X}, HL: {:#04X}, SP: {:#04X} FLAGS: {:?}",
                self.pc, opecode, self.a, self.bc, self.de, self.hl, self.sp, self.f,
            );

            self.debugger.step_run = (self.debugger.on_step)();
        }

        self.pc = self.pc.wrapping_add(1);

        let mnemonic = self.do_mnemonic(opecode)?;

        if debug {
            println!("{}", mnemonic);
        }

        self.bus.tick()?;

        Ok(())
    }

    pub fn b(&self) -> u8 {
        ((self.bc & 0xFF00) >> 8) as u8
    }

    pub fn c(&self) -> u8 {
        (self.bc & 0x00FF) as u8
    }

    pub fn d(&self) -> u8 {
        ((self.de & 0xFF00) >> 8) as u8
    }

    pub fn e(&self) -> u8 {
        (self.de & 0x00FF) as u8
    }

    pub fn h(&self) -> u8 {
        ((self.hl & 0xFF00) >> 8) as u8
    }

    pub fn l(&self) -> u8 {
        (self.hl & 0x00FF) as u8
    }

    fn set_b(&mut self, val: u8) {
        self.bc &= 0x00FF;
        self.bc |= ((val as u16) << 8) as u16;
    }

    fn set_c(&mut self, val: u8) {
        self.bc &= 0xFF00;
        self.bc |= val as u16;
    }

    fn set_d(&mut self, val: u8) {
        self.de &= 0x00FF;
        self.de |= ((val as u16) << 8) as u16;
    }

    fn set_e(&mut self, val: u8) {
        self.de &= 0xFF00;
        self.de |= val as u16;
    }

    fn set_h(&mut self, val: u8) {
        self.hl &= 0x00FF;
        self.hl |= ((val as u16) << 8) as u16;
    }

    fn set_l(&mut self, val: u8) {
        self.hl &= 0xFF00;
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
            7 => Ok(self.a),
            _ => bail!("unknown r8 {}", index),
        }
    }

    fn r8_str(&self, index: u8) -> String {
        match index {
            0 => "B".to_string(),
            1 => "C".to_string(),
            2 => "D".to_string(),
            3 => "E".to_string(),
            4 => "H".to_string(),
            5 => "L".to_string(),
            6 => format!("{:#04X}", self.hl),
            7 => "A".to_string(),
            _ => "?".to_string(),
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
                self.a = val;
                Ok(())
            }
            _ => bail!("unknown r8 {}", index),
        }
    }

    fn r16(&self, index: u8) -> Result<u16> {
        match index {
            0 => Ok(self.bc),
            1 => Ok(self.de),
            2 => Ok(self.hl),
            3 => Ok(self.sp),
            _ => bail!("unknown r16 {}", index),
        }
    }

    fn r16_str(&self, index: u8) -> String {
        match index {
            0 => "BC".to_string(),
            1 => "DE".to_string(),
            2 => "HL".to_string(),
            3 => "SP".to_string(),
            _ => "??".to_string(),
        }
    }

    fn set_r16(&mut self, index: u8, val: u16) -> Result<()> {
        match index {
            0 => {
                self.bc = val;
                Ok(())
            }
            1 => {
                self.de = val;
                Ok(())
            }
            2 => {
                self.hl = val;
                Ok(())
            }
            3 => {
                self.sp = val;
                Ok(())
            }
            _ => bail!("unknown r16 {}", index),
        }
    }

    fn carry_positive_n<
        T: Copy + PartialEq + From<bool> + Shr<T, Output = T> + BitAnd<T, Output = T>,
    >(
        &self,
        result: T,
        left: T,
        right: T,
        n: T,
    ) -> bool {
        let left_s = (left >> n) & n;
        let right_s = (right >> n) & n;
        let result_s = (result >> n) & n;

        (left_s == T::from(false) && right_s == T::from(false) && result_s == T::from(true))
            || (left_s == T::from(true) && right_s == T::from(true) && result_s == T::from(false))
    }

    fn carry_negative_n<
        T: Copy + PartialEq + From<bool> + Shr<T, Output = T> + BitAnd<T, Output = T>,
    >(
        &self,
        result: T,
        left: T,
        right: T,
        n: T,
    ) -> bool {
        let left_s = (left >> n) & n;
        let right_s = (right >> n) & n;
        let result_s = (result >> n) & n;

        (left_s == T::from(false) && right_s == T::from(true) && result_s == T::from(true))
            || (left_s == T::from(true) && right_s == T::from(false) && result_s == T::from(false))
    }

    fn carry_positive(&self, result: u8, left: u8, right: u8) -> bool {
        self.carry_positive_n(result, left, right, 7)
    }

    fn carry_negative(&self, result: u8, left: u8, right: u8) -> bool {
        self.carry_negative_n(result, left, right, 7)
    }

    fn half_carry_positive(&self, result: u8, left: u8, right: u8) -> bool {
        self.carry_positive_n(result, left, right, 3)
    }

    fn half_carry_negative(&self, result: u8, left: u8, right: u8) -> bool {
        self.carry_negative_n(result, left, right, 3)
    }

    fn carry_positive_16(&self, result: u16, left: u16, right: u16) -> bool {
        self.carry_positive_n(result, left, right, 15)
    }

    fn carry_negative_16(&self, result: u16, left: u16, right: u16) -> bool {
        self.carry_negative_n(result, left, right, 15)
    }

    fn half_carry_positive_16(&self, result: u16, left: u16, right: u16) -> bool {
        self.carry_positive_n(result, left, right, 11)
    }

    fn half_carry_negative_16(&self, result: u16, left: u16, right: u16) -> bool {
        self.carry_negative_n(result, left, right, 11)
    }

    #[bitmatch]
    fn do_mnemonic(&mut self, opecode: u8) -> Result<String> {
        #[bitmatch]
        match &opecode {
            // NOP
            "00000000" => self.nop(),
            // HALT
            "01110110" => self.halt(),
            // STOP
            "00010000" => self.stop(),
            // DI
            "11110011" => self.di(),
            // EI
            "11111011" => self.ei(),
            // LD r, r'
            // LD r, (HL)
            // LD (HL), r
            "01xxxyyy" => self.load_8_r_r(x, y),
            // LD r, n
            // LD (HL), n
            "00xxx110" => self.load_8_r_im8(x),
            // LD A, (BC)
            "00001010" => self.load_8_a_addr_bc(),
            // LD A, (DE)
            "00011010" => self.load_8_a_addr_de(),
            // LD (BC), A
            "00000010" => self.load_8_addr_bc_a(),
            // LD (DE), A
            "00010010" => self.load_8_addr_de_a(),
            // LD A, (nn)
            "11111010" => self.load_8_a_addr_im16(),
            // LD (nn), A
            "11101010" => self.load_8_addr_im16_a(),
            // LDH A, (C)
            "11110010" => self.load_8_a_addr_index_c(),
            // LDH (C), A
            "11100010" => self.load_8_addr_index_c_a(),
            // LDH A, (n)
            "11110000" => self.load_8_a_addr_index_im8(),
            // LDH (n), A
            "11100000" => self.load_8_addr_index_im8_a(),
            // LD A, (HL-)
            "00111010" => self.load_dec_8_a_addr_hl(),
            // LD (HL-), A
            "00110010" => self.load_dec_8_addr_hl_a(),
            // LD A, (HL+)
            "00101010" => self.load_inc_8_a_addr_hl(),
            // LD (HL+), A
            "00100010" => self.load_inc_8_addr_hl_a(),
            // LD rr, nn
            "00xx0001" => self.load_16_rr_im16(x),
            // LD (nn), SP
            "00001000" => self.load_16_addr_im16_sp(),
            // LD SP, HL
            "11111001" => self.load_16_sp_hl(),
            // PUSH rr
            "11xx0101" => self.push_16_rr(x),
            // POP rr
            "11xx0001" => self.pop_16_rr(x),
            // ADD A, r
            "10000xxx" => self.add_8_a_r(x),
            // ADD A, n
            "11000110" => self.add_8_a_im8(),
            // ADC A, r
            "10001xxx" => self.add_carry_8_a_r(x),
            // ADC A, n
            "11001110" => self.add_carry_8_a_im8(),
            // SUB A, r
            "10010xxx" => self.sub_8_a_r(x),
            // SUB n
            "11010110" => self.sub_8_a_im8(),
            // SBC A, r
            "10011xxx" => self.sub_carry_8_a_r(x),
            // SBC A, n
            "11011110" => self.sub_carry_8_a_im8(),
            // AND A, r
            "10100xxx" => self.and_8_a_r(x),
            // AND A, n
            "11100110" => self.and_8_a_im8(),
            // OR A, r
            "10110xxx" => self.or_8_a_r(x),
            // OR A, n
            "11110110" => self.or_8_a_im8(),
            // XOR A, r
            "10101xxx" => self.xor_8_a_r(x),
            // XOR A, n
            "11101110" => self.xor_8_a_im8(),
            // CP A, r
            "10111xxx" => self.cp_8_a_r(x),
            // CP A, n
            "11111110" => self.cp_8_a_im8(),
            // INC r
            "00xxx100" => self.inc_8_r(x),
            // DEC r
            "00xxx101" => self.dec_8_r(x),
            // ADD HL, rr
            "00xx1001" => self.add_16_hl_rr(x),
            // ADD SP, n
            "11101000" => self.add_16_sp_im8(),
            // INC rr
            "00xx0011" => self.inc_16_rr(x),
            // DEC rr
            "00xx1011" => self.dec_16_rr(x),
            // RLCA
            "00000111" => self.rlca_8(),
            // RLA
            "00010111" => self.rla_8(),
            // RRCA
            "00001111" => self.rrca_8(),
            // RRA
            "00011111" => self.rra_8(),
            // JP nn
            "11000011" => self.jp_16(),
            // JP NZ, nn
            "11000010" => self.jp_16_nz(),
            // JP Z, nn
            "11001010" => self.jp_16_z(),
            // JP NC, nn
            "11010010" => self.jp_16_nc(),
            // JP C, nn
            "11011010" => self.jp_16_c(),
            // JP (HL)
            "11101001" => self.jp_16_addr_hl(),
            // JR
            "00011000" => self.jr_8_im_8(),
            // JR NZ, nn
            "00100000" => self.jr_8_nz(),
            // JR Z, nn
            "00101000" => self.jr_8_z(),
            // JR NC, nn
            "00110000" => self.jr_8_nc(),
            // JR C, nn
            "00111000" => self.jr_8_c(),
            // CALL nn
            "11001101" => self.call_16(),
            // CALL NZ, nn
            "11000100" => self.call_16_nz(),
            // CALL Z, nn
            "11001100" => self.call_16_z(),
            // CALL NC, nn
            "11010100" => self.call_16_nc(),
            // CALL C, nn
            "11011100" => self.call_16_c(),
            // RST 00H~38H
            "11xxx111" => self.restart(x),
            // RET
            "11001001" => self.ret(),
            // RET NZ
            "11000000" => self.ret_nz(),
            // RET Z
            "11001000" => self.ret_z(),
            // RET NC
            "11010000" => self.ret_nc(),
            // RET C
            "11011000" => self.ret_c(),
            // RETI
            "11011001" => self.reti(),
            // CB Prefixed Instructions
            "11001011" => {
                let prefixed = self.bus.read(self.pc)?;
                self.pc = self.pc.wrapping_add(1);
                self.do_mnemonic_prefixed(prefixed)
            }
            _ => bail!("unimplemented opecode {:02X}", opecode),
        }
    }

    #[bitmatch]
    fn do_mnemonic_prefixed(&mut self, opecode: u8) -> Result<String> {
        #[bitmatch]
        match &opecode {
            // SWAP r
            "01000xxx" => self.swap_8_r(x),
            // DAA
            "00100111" => self.decimal_adjust_8_a(),
            // CPL
            "00101111" => self.complement_8_a(),
            // CCF
            "00111111" => self.complement_carry(),
            // SCF
            "00110111" => self.set_carry_flag(),
            // RLC r
            "00000xxx" => self.rlc_8_r(x),
            // RL r
            "00010xxx" => self.rl_8_r(x),
            // RRC r
            "00001xxx" => self.rrc_8_r(x),
            // RR r
            "00011xxx" => self.rr_8_r(x),
            // SLA r
            "00100xxx" => self.sla_8_r(x),
            // SRA r
            "00101xxx" => self.sra_8_r(x),
            // SRL r
            "00111xxx" => self.srl_8_r(x),
            // BIT b, r
            "01000xxx" => self.bit_8_im_bit_r(x),
            // SET b, r
            "11000xxx" => self.set_8_im_bit_r(x),
            // RES b, r
            "10000xxx" => self.reset_8_im_bit_r(x),
            _ => bail!("unimplemented prefixed opecode {:?}", opecode),
        }
    }

    pub fn nop(&self) -> Result<String> {
        Ok("NOP".to_string())
    }

    pub fn halt(&mut self) -> Result<String> {
        // unimplemented!("停止する");

        Ok("HALT".to_string())
    }

    pub fn stop(&mut self) -> Result<String> {
        // unimplemented!("停止して、LCDそのまま");

        Ok("STOP".to_string())
    }

    pub fn di(&mut self) -> Result<String> {
        // unimplemented!("直後の命令実行後に割り込み中止");

        Ok("DI".to_string())
    }

    pub fn ei(&mut self) -> Result<String> {
        // unimplemented!("直後の命令実行後に割り込み再開");

        Ok("EI".to_string())
    }

    pub fn load_8_r_im8(&mut self, index: u8) -> Result<String> {
        let val = self.bus.read(self.pc)?;

        self.pc = self.pc.wrapping_add(1);

        self.set_r8(index, val)?;

        Ok(format!("LD {}, n: n={:02X}", self.r8_str(index), val))
    }

    pub fn load_8_r_r(&mut self, left: u8, right: u8) -> Result<String> {
        let val = self.r8(right)?;
        self.set_r8(left, val)?;

        Ok(format!(
            "LD {}, {}: {}={:02X}",
            self.r8_str(left),
            self.r8_str(right),
            self.r8_str(left),
            val
        ))
    }

    pub fn load_8_a_addr_bc(&mut self) -> Result<String> {
        let val = self.bus.read(self.bc)?;
        self.a = val;

        Ok(format!("LD A, (BC): (BC)=({:04X})={:02X}", self.bc, val))
    }

    pub fn load_8_a_addr_de(&mut self) -> Result<String> {
        let val = self.bus.read(self.de)?;
        self.a = val;

        Ok(format!("LD A, (DE): (DE)=({:04X})={:04X}", self.de, val))
    }

    pub fn load_8_addr_bc_a(&mut self) -> Result<String> {
        self.bus.write(self.bc, self.a)?;

        Ok(format!(
            "LD (BC), A: (BC)=({:04X}), A={:02X}",
            self.bc, self.a
        ))
    }

    pub fn load_8_addr_de_a(&mut self) -> Result<String> {
        self.bus.write(self.de, self.a)?;

        Ok(format!(
            "LD (DE), A: (DE)=({:04X}), A={:02X}",
            self.de, self.a
        ))
    }

    pub fn load_8_a_addr_im16(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        let val = self.bus.read(addr)?;
        self.a = val;

        Ok(format!("LD A, (nn): (nn)=({:04X})={:02X}", addr, val,))
    }

    pub fn load_8_addr_im16_a(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        let val = self.bus.read(addr)?;
        self.bus.write(addr, val)?;

        Ok(format!("LD (nn), A: (nn)=({:04X}), A={:02X}", addr, val))
    }

    pub fn load_8_a_addr_index_c(&mut self) -> Result<String> {
        let index = self.c();
        let addr = 0xFF00 + index as u16;
        let val = self.bus.read(addr)?;
        self.a = val;

        Ok(format!(
            "LDH A, (C): (C)=({:02X})=({:04X})={:02X}",
            index, addr, val
        ))
    }

    pub fn load_8_addr_index_c_a(&mut self) -> Result<String> {
        let index = self.c();
        let addr = 0xFF00 + index as u16;
        self.bus.write(addr, self.a)?;

        Ok(format!(
            "LDH (C), A: (C)=({:02X})=({:04X})={:02X}",
            index, addr, self.a
        ))
    }

    pub fn load_8_a_addr_index_im8(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let addr = 0xFF00 + index as u16;
        let val = self.bus.read(addr)?;
        self.a = val;

        Ok(format!(
            "LDH A, (n): (n)=({:02X})=({:04X})={:02X}",
            index, addr, val
        ))
    }

    pub fn load_8_addr_index_im8_a(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let addr = 0xFF00 + index as u16;
        self.bus.write(addr, self.a)?;

        Ok(format!(
            "LDH (n), A: (n)=({:02X})=({:04X}), A={:02X}",
            index, addr, self.a,
        ))
    }

    pub fn load_dec_8_a_addr_hl(&mut self) -> Result<String> {
        let val = self.bus.read(self.hl)?;
        self.hl = self.hl.wrapping_sub(1);
        self.a = val;

        Ok(format!(
            "LD A, (HL-): (HL)=({:04X})={:02X}, (HL-)=({:04X})",
            self.hl.wrapping_add(1),
            val,
            self.hl
        ))
    }

    pub fn load_dec_8_addr_hl_a(&mut self) -> Result<String> {
        self.bus.write(self.hl, self.a)?;
        self.hl = self.hl.wrapping_sub(1);

        Ok(format!(
            "LD (HL-), A: (HL)=({:04X}), (HL-)=({:04X}), A={:02X}",
            self.hl.wrapping_add(1),
            self.hl,
            self.a,
        ))
    }

    pub fn load_inc_8_a_addr_hl(&mut self) -> Result<String> {
        let val = self.bus.read(self.hl)?;
        self.hl = self.hl.wrapping_add(1);
        self.a = val;

        Ok(format!(
            "LD A, (HL+): (HL)=({:04X})={:02X}, (HL+)=({:04X})",
            self.hl.wrapping_sub(1),
            val,
            self.hl,
        ))
    }

    pub fn load_inc_8_addr_hl_a(&mut self) -> Result<String> {
        self.bus.write(self.hl, self.a)?;
        self.hl = self.hl.wrapping_add(1);

        Ok(format!(
            "LD (HL+), A: (HL)=({:04X}), (HL+)=({:04X}), A={:02X}",
            self.hl.wrapping_sub(1),
            self.hl,
            self.a
        ))
    }

    pub fn load_16_rr_im16(&mut self, index: u8) -> Result<String> {
        let val = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        self.set_r16(index, val)?;

        Ok(format!("LD {}, nn: nn={:04X}", self.r16_str(index), val,))
    }

    pub fn load_16_addr_im16_sp(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        let val = self.bus.read_word(addr)?;
        self.sp = val;

        Ok(format!("LD (nn), sp: (nn)=({:04X}), SP={:04X}", addr, val))
    }

    pub fn load_16_sp_hl(&mut self) -> Result<String> {
        self.sp = self.hl;

        Ok(format!("LD SP, HL: HL={:04X}", self.hl))
    }

    pub fn push_16_rr(&mut self, index: u8) -> Result<String> {
        let val = self.r16(index)?;
        self.bus.write_word(self.sp, val)?;
        self.sp = self.sp.wrapping_sub(2);

        Ok(format!("PUSH {}: {0}={:04X}", self.r16_str(index), val))
    }

    pub fn pop_16_rr(&mut self, index: u8) -> Result<String> {
        self.sp = self.sp.wrapping_add(2);
        let val = self.bus.read_word(self.sp)?;
        self.set_r16(index, val)?;

        Ok(format!("POP {}: data={:04X}", self.r16_str(index), val))
    }

    pub fn add_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left.wrapping_add(right);

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(self.half_carry_positive(result, left, right));
        self.f.set_c(self.carry_positive(result, left, right));

        Ok(format!(
            "ADD A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn add_8_a_im8(&mut self) -> Result<String> {
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let left = self.a;
        let result = left.wrapping_add(right);

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(self.half_carry_positive(result, left, right));
        self.f.set_c(self.carry_positive(result, left, right));

        Ok(format!("ADD A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn add_carry_8_a_r(&mut self, index: u8) -> Result<String> {
        let c = self.f.c() as u8;
        let right = self.r8(index)?;
        let left = self.a;
        let result1 = left.wrapping_add(right);
        let result2 = result1.wrapping_add(c);

        let c1 = self.carry_positive(result1, left, right);
        let h1 = self.half_carry_positive(result1, left, right);
        let c2 = self.carry_positive(result2, result1, c);
        let h2 = self.half_carry_positive(result2, result1, c);

        self.a = result2;

        self.f.set_z(result2 == 0);
        self.f.set_n(false);
        self.f.set_h(h1 || h2);
        self.f.set_c(c1 || c2);

        Ok(format!(
            "ADC A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right,
        ))
    }

    pub fn add_carry_8_a_im8(&mut self) -> Result<String> {
        let c = self.f.c() as u8;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let left = self.a;
        let result1 = left.wrapping_add(right);
        let result2 = result1.wrapping_add(c);

        let c1 = self.carry_positive(result1, left, right);
        let h1 = self.half_carry_positive(result1, left, right);
        let c2 = self.carry_positive(result2, result1, c);
        let h2 = self.half_carry_positive(result2, result1, c);

        self.a = result2;

        self.f.set_z(result2 == 0);
        self.f.set_n(false);
        self.f.set_h(h1 || h2);
        self.f.set_c(c1 || c2);

        Ok(format!("ADC A, n: A={:02X}, n={:02X}", left, right,))
    }

    pub fn sub_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left.wrapping_sub(right);

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(true);
        self.f.set_h(self.half_carry_negative(result, left, right));
        self.f.set_c(self.carry_negative(result, left, right));

        Ok(format!(
            "SUB A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn sub_8_a_im8(&mut self) -> Result<String> {
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left.wrapping_sub(right);

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(true);
        self.f.set_h(self.half_carry_negative(result, left, right));
        self.f.set_c(self.carry_negative(result, left, right));

        Ok(format!("SUB A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn sub_carry_8_a_r(&mut self, index: u8) -> Result<String> {
        let c = self.f.c() as u8;
        let left = self.a;
        let right = self.r8(index)?;
        let result1 = left.wrapping_sub(right);
        let result2 = result1.wrapping_sub(c);

        self.a = result2;

        let c1 = self.carry_negative(result1, left, right);
        let h1 = self.half_carry_negative(result1, left, right);
        let c2 = self.carry_negative(result2, result1, c);
        let h2 = self.half_carry_negative(result2, result1, c);

        self.f.set_z(result2 == 0);
        self.f.set_n(true);
        self.f.set_h(h1 || h2);
        self.f.set_c(c1 || c2);

        Ok(format!(
            "SBC A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn sub_carry_8_a_im8(&mut self) -> Result<String> {
        let c = self.f.c() as u8;
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result1 = left.wrapping_sub(right);
        let result2 = result1.wrapping_sub(c);

        self.a = result2;

        let c1 = self.carry_negative(result1, left, right);
        let h1 = self.half_carry_negative(result1, left, right);
        let c2 = self.carry_negative(result2, result1, c);
        let h2 = self.half_carry_negative(result2, result1, c);

        self.f.set_z(result2 == 0);
        self.f.set_n(true);
        self.f.set_h(h1 || h2);
        self.f.set_c(c1 || c2);

        Ok(format!("SBC A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn and_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left & right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(true);
        self.f.set_c(false);

        Ok(format!(
            "AND A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn and_8_a_im8(&mut self) -> Result<String> {
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left & right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(true);
        self.f.set_c(false);

        Ok(format!("AND A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn or_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left | right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(false);

        Ok(format!(
            "OR A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn or_8_a_im8(&mut self) -> Result<String> {
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left | right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(false);

        Ok(format!("OR A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn xor_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left ^ right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(false);

        Ok(format!(
            "XOR A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn xor_8_a_im8(&mut self) -> Result<String> {
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left ^ right;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(false);

        Ok(format!("XOR A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn cp_8_a_r(&mut self, index: u8) -> Result<String> {
        let left = self.a;
        let right = self.r8(index)?;
        let result = left.wrapping_sub(left);

        self.f.set_z(result == 0);
        self.f.set_n(true);
        self.f.set_h(self.half_carry_negative(result, left, right));
        self.f.set_c(self.carry_negative(result, left, right));

        Ok(format!(
            "CP A, {}: A={:02X}, {0}={:02X}",
            self.r8_str(index),
            left,
            right
        ))
    }

    pub fn cp_8_a_im8(&mut self) -> Result<String> {
        let left = self.a;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left.wrapping_sub(right);

        self.f.set_z(result == 0);
        self.f.set_n(true);
        self.f.set_h(self.half_carry_negative(result, left, right));
        self.f.set_c(self.carry_negative(result, left, right));

        Ok(format!("CP A, n: A={:02X}, n={:02X}", left, right))
    }

    pub fn inc_8_r(&mut self, index: u8) -> Result<String> {
        let left = self.r8(index)?;
        let right = 1;
        let result = left.wrapping_add(right);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(self.half_carry_positive(result, left, right));
        self.f.set_c(self.carry_positive(result, left, right));

        Ok(format!("INC {}: {0}={:02X}", self.r8_str(index), left))
    }

    pub fn dec_8_r(&mut self, index: u8) -> Result<String> {
        let left = self.r8(index)?;
        let right = 1;
        let result = left.wrapping_sub(right);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(true);
        self.f.set_h(self.half_carry_negative(result, left, right));
        self.f.set_c(self.carry_negative(result, left, right));

        Ok(format!("DEC {}: {0}={:02X}", self.r8_str(index), left))
    }

    pub fn add_16_hl_rr(&mut self, index: u8) -> Result<String> {
        let left = self.hl;
        let right = self.r16(index)?;
        let result = left.wrapping_add(right);

        self.hl = result;

        self.f.set_n(false);
        self.f
            .set_h(self.half_carry_positive_16(result, left, right));
        self.f.set_c(self.carry_positive_16(result, left, right));

        Ok(format!(
            "ADD HL, {}: HL={:04X}, {0}={:04X}",
            self.r16_str(index),
            left,
            right
        ))
    }

    pub fn add_16_sp_im8(&mut self) -> Result<String> {
        let left = self.sp;
        let right = self.bus.read(self.pc)? as i8 as u16;
        self.pc = self.pc.wrapping_add(1);
        let result = left.wrapping_add(right);

        self.sp = result;

        self.f.set_z(false);
        self.f.set_n(false);
        self.f
            .set_h(self.half_carry_positive_16(result, left, right));
        self.f.set_c(self.carry_positive_16(result, left, right));

        Ok(format!("ADD SP, n: SP={:04X}, n={:02X}", left, right))
    }

    pub fn inc_16_rr(&mut self, index: u8) -> Result<String> {
        let left = self.r16(index)?;
        let right = 1;
        let result = left.wrapping_add(right);

        self.set_r16(index, result)?;

        Ok(format!("INC {}: {0}={:04X}", self.r16_str(index), left))
    }

    pub fn dec_16_rr(&mut self, index: u8) -> Result<String> {
        let left = self.r16(index)?;
        let right = 1;
        let result = left.wrapping_sub(right);

        self.set_r16(index, result)?;

        Ok(format!("DEC {}: {0}={:04X}", self.r16_str(index), left))
    }

    pub fn rlca_8(&mut self) -> Result<String> {
        let val = self.a;
        let c = (val >> 7) & 1;
        let result = val << 1;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!("RLCA: A={:02X}, #={:02X}", val, result))
    }

    pub fn rla_8(&mut self) -> Result<String> {
        let val = self.a;
        let c = (val >> 7) & 1;
        let result = val << 1 | self.f.c() as u8;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!("RLA: A={:02X}, #={:02X}", val, result))
    }

    pub fn rrca_8(&mut self) -> Result<String> {
        let val = self.a;
        let c = val & 1;
        let result = val >> 1;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!("RRCA: A={:02X}, #={:02X}", val, result))
    }

    pub fn rra_8(&mut self) -> Result<String> {
        let val = self.a;
        let c = val & 1;
        let result = val >> 1 | ((self.f.c() as u8) << 7);

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!("RRA: A={:02X}, #={:02X}", val, result))
    }

    pub fn rlc_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = (val >> 7) & 1;
        let result = val.rotate_left(1);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "RLC {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn rl_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = (val >> 7) & 1;
        let result = val << 1 | self.f.c() as u8;

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "RL {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn rrc_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = val & 1;
        let result = val.rotate_right(1);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "RRC {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn rr_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = val & 1;
        let result = val >> 1 | ((self.f.c() as u8) << 7);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "RR {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn sla_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = (val >> 7) & 1;
        let result = val << 1;

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "SLA {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn sra_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = val & 1;
        let result = val >> 1 | (val & 0b10000000);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "SRA {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn srl_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let c = val & 1;
        let result = val >> 1;

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(c == 1);

        Ok(format!(
            "SRL {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn bit_8_im_bit_r(&mut self, index: u8) -> Result<String> {
        let left = self.r8(index)?;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = (left >> right) & 1;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(true);

        Ok(format!(
            "BIT b, {}: b={}, {0}={:02X}, #={:02X}",
            self.r8_str(index),
            right,
            left,
            result
        ))
    }

    pub fn set_8_im_bit_r(&mut self, index: u8) -> Result<String> {
        let left = self.r8(index)?;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left | (1 << right);

        self.set_r8(index, result)?;

        Ok(format!(
            "SET b, {}: b={}, {0}={:02X}, #={:02X}",
            self.r8_str(index),
            right,
            left,
            result
        ))
    }

    pub fn reset_8_im_bit_r(&mut self, index: u8) -> Result<String> {
        let left = self.r8(index)?;
        let right = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        let result = left & !(1 << right);

        self.set_r8(index, result)?;

        Ok(format!(
            "RES b, {}: b={}, {0}={:02X}, #={:02X}",
            self.r8_str(index),
            right,
            left,
            result
        ))
    }

    pub fn jp_16(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = addr;

        Ok(format!("JP nn: nn={:04X}", addr))
    }

    pub fn jp_16_nz(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if !self.f.z() {
            self.pc = addr;
        }

        Ok(format!("JP NZ, nn: NZ={}, nn={:04X}", !self.f.z(), addr))
    }

    pub fn jp_16_z(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if self.f.z() {
            self.pc = addr;
        }

        Ok(format!("JP Z, nn: Z={}, nn={:04X}", self.f.z(), addr))
    }

    pub fn jp_16_nc(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if !self.f.c() {
            self.pc = addr;
        }

        Ok(format!("JP NC, nn: NC={}, nn={:04X}", !self.f.c(), addr))
    }

    pub fn jp_16_c(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if self.f.c() {
            self.pc = addr;
        }

        Ok(format!("JP C, nn: C={}, nn={:04X}", self.f.c(), addr))
    }

    pub fn jp_16_addr_hl(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.hl)?;
        self.pc = addr;

        Ok(format!("JP (HL): (HL)=({:04X})={:04X}", self.hl, addr))
    }

    pub fn jr_8_im_8(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);
        self.pc = self.pc.wrapping_add(index as i8 as u16);

        Ok(format!("JR n: n={}", index))
    }

    pub fn jr_8_nz(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);

        if !self.f.z() {
            self.pc = self.pc.wrapping_add(index as i8 as u16);
        }

        Ok(format!("JR NZ, n: NZ={}, n={}", !self.f.z(), index))
    }

    pub fn jr_8_z(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);

        if self.f.z() {
            self.pc = self.pc.wrapping_add(index as i8 as u16);
        }

        Ok(format!("JR Z, n: Z={}, n={}", self.f.z(), index))
    }

    pub fn jr_8_nc(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);

        if !self.f.c() {
            self.pc = self.pc.wrapping_add(index as i8 as u16);
        }

        Ok(format!("JR NC, n: NC={}, n={}", !self.f.c(), index))
    }

    pub fn jr_8_c(&mut self) -> Result<String> {
        let index = self.bus.read(self.pc)?;
        self.pc = self.pc.wrapping_add(1);

        if self.f.c() {
            self.pc = self.pc.wrapping_add(index as i8 as u16);
        }

        Ok(format!("JR C, n: C={}, n={}", self.f.c(), index))
    }

    pub fn call_16(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);
        self.bus.write_word(self.sp, self.pc)?;
        self.sp = self.sp.wrapping_sub(2);
        self.pc = addr;

        Ok(format!("CALL nn: nn={:04X}", addr))
    }

    pub fn call_16_nz(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if !self.f.z() {
            self.bus.write_word(self.sp, self.pc)?;
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!("CALL NZ, nn: NZ={}, nn={:04X}", !self.f.z(), addr))
    }

    pub fn call_16_z(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if self.f.z() {
            self.bus.write_word(self.sp, self.pc)?;
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!("CALL Z, nn: Z={}, nn={:04X}", self.f.z(), addr))
    }

    pub fn call_16_nc(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if !self.f.c() {
            self.bus.write_word(self.sp, self.pc)?;
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!("CALL NC, nn: NC={}, nn={:04X}", !self.f.c(), addr))
    }

    pub fn call_16_c(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.pc)?;
        self.pc = self.pc.wrapping_add(2);

        if self.f.c() {
            self.bus.write_word(self.sp, self.pc)?;
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!("CALL C, nn: C={}, nn={:04X}", self.f.c(), addr))
    }

    pub fn restart(&mut self, param: u8) -> Result<String> {
        let addr = param as u16;
        self.bus.write_word(self.sp, self.pc)?;
        self.sp = self.sp.wrapping_sub(2);
        self.pc = addr;

        Ok(format!("RST nn: nn={:04X}", addr))
    }

    pub fn ret(&mut self) -> Result<String> {
        self.sp = self.sp.wrapping_sub(2);
        let addr = self.bus.read_word(self.sp)?;
        self.pc = addr;

        Ok(format!(
            "RET: (SP)=({:04X})={:04X}",
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn ret_nz(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.sp)?;

        if !self.f.z() {
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!(
            "RET NZ: NZ={}, (SP)=({:04X})={:04X}",
            !self.f.z(),
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn ret_z(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.sp)?;

        if self.f.z() {
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!(
            "RET Z: Z={}, (SP)=({:04X})={:04X}",
            self.f.z(),
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn ret_nc(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.sp)?;

        if !self.f.c() {
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!(
            "RET NC: NC={}, (SP)=({:04X})={:04X}",
            !self.f.c(),
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn ret_c(&mut self) -> Result<String> {
        let addr = self.bus.read_word(self.sp)?;

        if self.f.c() {
            self.sp = self.sp.wrapping_sub(2);
            self.pc = addr;
        }

        Ok(format!(
            "RET C: C={}, (SP)=({:04X})={:04X}",
            self.f.c(),
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn reti(&mut self) -> Result<String> {
        self.sp = self.sp.wrapping_sub(2);
        let addr = self.bus.read_word(self.sp)?;
        self.pc = addr;

        unimplemented!("割り込みを再開");

        Ok(format!(
            "RETI: (SP)=({:04X})={:04X}",
            self.sp.wrapping_add(2),
            addr
        ))
    }

    pub fn swap_8_r(&mut self, index: u8) -> Result<String> {
        let val = self.r8(index)?;
        let high = val & 0xF0;
        let low = val & 0x0F;
        let result = (high >> 4) | (low << 4);

        self.set_r8(index, result)?;

        self.f.set_z(result == 0);
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(false);

        Ok(format!(
            "SWAP {}: {0}={:02X}, #={:02X}",
            self.r8_str(index),
            val,
            result
        ))
    }

    pub fn decimal_adjust_8_a(&mut self) -> Result<String> {
        let val = self.a;

        unimplemented!("BCDに変換");

        let result = val;
        let c = false;

        self.a = result;

        self.f.set_z(result == 0);
        self.f.set_h(false);
        self.f.set_c(c);

        Ok(format!("DAA: A={:02X}, #={:02X}", val, result))
    }

    pub fn complement_8_a(&mut self) -> Result<String> {
        let val = self.a;
        let result = !val;

        self.a = result;
        self.f.set_n(true);
        self.f.set_h(true);

        Ok(format!("CPL: A={:02X}, #={:02X}", val, result))
    }

    pub fn complement_carry(&mut self) -> Result<String> {
        let c = self.f.c();
        let result = !c;

        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(result);

        Ok(format!("CCF: C={}, #={}", c, result))
    }

    pub fn set_carry_flag(&mut self) -> Result<String> {
        self.f.set_n(false);
        self.f.set_h(false);
        self.f.set_c(true);

        Ok("SCF".to_string())
    }
}
