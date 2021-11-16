use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::thread::sleep;
use std::time::Duration;

enum Pin {
    Rd = 20,
    Wr = 3,
    Cs = 26,
    Cs2 = 2,
    Rst = 23,

    Addr0 = 16,
    Addr1 = 19,
    Addr2 = 13,
    Addr3 = 12,
    Addr4 = 6,
    Addr5 = 5,
    Addr6 = 25,
    Addr7 = 24,
    Addr8 = 22,
    Addr9 = 27,
    Addr10 = 18,
    Addr11 = 17,
    Addr12 = 15,
    Addr13 = 14,
    Addr14 = 4,
    Addr15 = 21,
}

const MCP_BASE: u8 = 123;
const DEV_ID: u8 = 0;

const MCP23X08_IODIR: u8 = 0x00;
const MCP23X08_IPOL: u8 = 0x01;
const MCP23X08_GPINTEN: u8 = 0x02;
const MCP23X08_DEFVAL: u8 = 0x03;
const MCP23X08_INTCON: u8 = 0x04;
const MCP23X08_IOCON: u8 = 0x05;
const MCP23X08_GPPU: u8 = 0x06;
const MCP23X08_INTF: u8 = 0x07;
const MCP23X08_INTCAP: u8 = 0x08;
const MCP23X08_GPIO: u8 = 0x09;
const MCP23X08_OLAT: u8 = 0x0A;

const CS_WAIT: u64 = 3;
const RD_WAIT: u64 = 4;
const WR_WAIT_BEFORE: u64 = 1;
const WR_WAIT_AFTER: u64 = 5;

const CMD_WRITE: u8 = 0x40;
const CMD_READ: u8 = 0x41;

const IOCON_INIT: u8 = 0x20;

pub struct CubicStyleBoard {
    gpio: Gpio,
    spi: Spi,
    prev: u8,

    rd: OutputPin,
    wr: OutputPin,
    cs: OutputPin,
    rst: OutputPin,

    addr: [OutputPin; 16],
}

impl CubicStyleBoard {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;

        let rd = (&gpio).get(Pin::Rd as u8)?.into_output();
        let wr = (&gpio).get(Pin::Wr as u8)?.into_output();
        let cs = (&gpio).get(Pin::Cs as u8)?.into_output();
        let rst = (&gpio).get(Pin::Rst as u8)?.into_output();
        let addr = [
            (&gpio).get(Pin::Addr0 as u8)?.into_output(),
            (&gpio).get(Pin::Addr1 as u8)?.into_output(),
            (&gpio).get(Pin::Addr2 as u8)?.into_output(),
            (&gpio).get(Pin::Addr3 as u8)?.into_output(),
            (&gpio).get(Pin::Addr4 as u8)?.into_output(),
            (&gpio).get(Pin::Addr5 as u8)?.into_output(),
            (&gpio).get(Pin::Addr6 as u8)?.into_output(),
            (&gpio).get(Pin::Addr7 as u8)?.into_output(),
            (&gpio).get(Pin::Addr8 as u8)?.into_output(),
            (&gpio).get(Pin::Addr9 as u8)?.into_output(),
            (&gpio).get(Pin::Addr10 as u8)?.into_output(),
            (&gpio).get(Pin::Addr11 as u8)?.into_output(),
            (&gpio).get(Pin::Addr12 as u8)?.into_output(),
            (&gpio).get(Pin::Addr13 as u8)?.into_output(),
            (&gpio).get(Pin::Addr14 as u8)?.into_output(),
            (&gpio).get(Pin::Addr15 as u8)?.into_output(),
        ];

        Ok(Self {
            gpio,
            spi: Spi::new(Bus::Spi0, SlaveSelect::Ss1, 4000000, Mode::Mode0)?,
            prev: 0,
            rd,
            wr,
            cs,
            rst,
            addr,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.write_mcp_byte(MCP23X08_IOCON, IOCON_INIT)?;

        self.prev = self.read_mcp_byte(MCP23X08_OLAT)?;

        self.rd.set_high();
        self.wr.set_high();
        self.rst.set_high();
        self.cs.set_high();

        Ok(())
    }

    pub fn set_addr(&mut self, addr: u16) {
        for i in 0..16 {
            let pin = &mut self.addr[i];
            if addr & (1 << i) > 0 {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        self.set_write(false);
        self.set_read(true);
        self.set_cs(true);

        let mut data: u8 = 0x00;

        for i in 0..8 {
            if self.read_mcp_pin(i + MCP_BASE)? {
                data |= 1 << i;
            }
        }

        self.set_read(false);
        self.set_cs(false);

        Ok(data)
    }

    pub fn write_byte(&mut self, val: u8) -> Result<()> {
        self.set_read(false);
        self.set_cs(true);

        for i in 0..8 {
            let bit = val & (1 << i) > 0;

            self.write_mcp_pin(i + MCP_BASE, bit)?;
        }

        self.set_write(true);
        self.set_write(false);
        self.set_cs(false);

        Ok(())
    }

    fn set_write(&mut self, val: bool) {
        sleep(Duration::from_micros(WR_WAIT_BEFORE));

        if val {
            self.wr.set_low();
        } else {
            self.wr.set_high();
        }

        sleep(Duration::from_micros(WR_WAIT_AFTER));
    }

    fn set_read(&mut self, val: bool) {
        if val {
            self.rd.set_low();
        } else {
            self.rd.set_high();
        }

        sleep(Duration::from_micros(RD_WAIT));
    }

    fn set_cs(&mut self, val: bool) {
        if val {
            self.cs.set_low();
        } else {
            self.cs.set_high();
        }

        sleep(Duration::from_micros(CS_WAIT));
    }

    fn write_mcp_pin(&mut self, pin: u8, val: bool) -> Result<()> {
        let bit = 1 << ((pin - MCP_BASE) & 7);

        let mut prev = self.prev;

        if val {
            prev |= bit;
        } else {
            prev &= !bit;
        }

        self.write_mcp_byte(MCP23X08_GPIO, prev)?;

        self.prev = prev;

        Ok(())
    }

    fn read_mcp_pin(&mut self, pin: u8) -> Result<bool> {
        let mask = 1 << ((pin - MCP_BASE) & 7);
        let val = self.read_mcp_byte(MCP23X08_GPIO)? & mask > 0;

        Ok(val)
    }

    fn write_mcp_byte(&mut self, reg: u8, val: u8) -> Result<()> {
        let mut data: [u8; 4] = [0; 4];
        data[0] = CMD_WRITE | ((DEV_ID & 7) << 1);
        data[1] = reg;
        data[2] = val;

        self.spi.write(&data[..])?;

        Ok(())
    }

    fn read_mcp_byte(&mut self, reg: u8) -> Result<u8> {
        let mut data: [u8; 4] = [0; 4];
        data[0] = CMD_READ | ((DEV_ID & 7) << 1);
        data[1] = reg;

        let mut buffer: [u8; 4] = [0; 4];

        self.spi.transfer(&mut buffer, &data[..])?;

        Ok(buffer[2])
    }
}
