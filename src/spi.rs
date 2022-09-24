//! BME280 driver for sensors attached via SPI.

use embedded_hal::digital::blocking::OutputPin;
use embedded_hal::spi::blocking::Transfer;
use crate::Interface;

use super::{
    Error,
    BME280_H_CALIB_DATA_LEN, BME280_P_T_CALIB_DATA_LEN, BME280_P_T_H_DATA_LEN,
};

/// Register access functions for SPI
#[derive(Debug, Default)]
pub struct SPIInterface<SPI, CS> {
    /// concrete SPI device implementation
    spi: SPI,
    /// chip-select pin
    cs: CS,
}

impl<SPI, CS>  SPIInterface<SPI, CS>
    where
        SPI: Transfer<u8>,
        CS: OutputPin,
{
    /// Create a new BME280 struct
    pub fn new(spi: SPI, mut cs: CS) -> Result<Self, Error<SPIError<SPI::Error, CS::Error>>> {
        // Deassert chip-select.
        cs.set_high().map_err(|e| Error::Bus(SPIError::Pin(e)))?;

        Ok(SPIInterface { spi, cs })
    }
}

impl<SPI, CS> Interface for SPIInterface<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    type Error = SPIError<SPI::Error, CS::Error>;

    fn read_register(&mut self, register: u8) -> Result<u8, Error<Self::Error>> {
        let mut result = [0u8];
        self.read_any_register(register, &mut result)?;
        Ok(result[0])
    }

    fn read_data(
        &mut self,
        register: u8,
    ) -> Result<[u8; BME280_P_T_H_DATA_LEN], Error<Self::Error>> {
        let mut data = [0; BME280_P_T_H_DATA_LEN];
        self.read_any_register(register, &mut data)?;
        Ok(data)
    }

    fn read_pt_calib_data(
        &mut self,
        register: u8,
    ) -> Result<[u8; BME280_P_T_CALIB_DATA_LEN], Error<Self::Error>> {
        let mut data = [0; BME280_P_T_CALIB_DATA_LEN];
        self.read_any_register(register, &mut data)?;
        Ok(data)
    }

    fn read_h_calib_data(
        &mut self,
        register: u8,
    ) -> Result<[u8; BME280_H_CALIB_DATA_LEN], Error<Self::Error>> {
        let mut data = [0; BME280_H_CALIB_DATA_LEN];
        self.read_any_register(register, &mut data)?;
        Ok(data)
    }

    fn write_register(&mut self, register: u8, payload: u8) -> Result<(), Error<Self::Error>> {
        self.cs
            .set_low()
            .map_err(|e| Error::Bus(SPIError::Pin(e)))?;
        // If the first bit is 0, the register is written.
        let transfer = [register & 0x7f, payload];
        self.spi
            .transfer(&mut [], &transfer)
            .map_err(|e| Error::Bus(SPIError::SPI(e)))?;
        self.cs
            .set_high()
            .map_err(|e| Error::Bus(SPIError::Pin(e)))?;
        Ok(())
    }
}

impl<SPI, CS> SPIInterface<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    fn read_any_register(
        &mut self,
        register: u8,
        data: &mut [u8],
    ) -> Result<(), Error<SPIError<SPI::Error, CS::Error>>> {
        self.cs
            .set_low()
            .map_err(|e| Error::Bus(SPIError::Pin(e)))?;
        self.spi
            .transfer(data, &[register])
            .map_err(|e| Error::Bus(SPIError::SPI(e)))?;
        self.cs
            .set_high()
            .map_err(|e| Error::Bus(SPIError::Pin(e)))?;
        Ok(())
    }
}

/// Error which occurred during an SPI transaction
#[derive(Clone, Copy, Debug)]
pub enum SPIError<SPIE, PinE> {
    /// The SPI implementation returned an error
    SPI(SPIE),
    /// The GPIO implementation returned an error which changing the chip-select pin state
    Pin(PinE),
}
