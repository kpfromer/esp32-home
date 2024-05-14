use embedded_hal_async::i2c::I2c;

/// Based on https://crates.io/crates/dht20
pub struct Dht20<I2C, E>
where
    I2C: I2c<Error = E>,
{
    i2c: I2C,
    address: u8,
}

#[derive(Debug, Clone)]
pub struct Reading {
    pub temp: f32,
    pub hum: f32,
}

#[derive(Debug)]
pub enum Error<E> {
    I2cError(E),
    ReadToFast,
}

impl<I2CDevice, E> Dht20<I2CDevice, E>
where
    I2CDevice: I2c<Error = E>,
{
    pub fn new(i2c: I2CDevice, address: u8) -> Self {
        Self { i2c, address }
    }

    pub async fn read(&mut self) -> Result<Reading, E> {
        self.reset().await?;
        // request reading
        self.write_data(&[0xAC, 0x33, 0]).await?;
        // read data
        let data = self.read_data().await?;
        // convert values
        let mut raw = (data[1] as u32) << 8;
        raw += data[2] as u32;
        raw <<= 4;
        raw += (data[3] >> 4) as u32;
        let hum = raw as f32 * 9.5367431640625e-5; // ==> / 1048576.0 * 100%;

        let mut raw = (data[3] & 0x0F) as u32;
        raw <<= 8;
        raw += data[4] as u32;
        raw <<= 8;
        raw += data[5] as u32;
        let temp = raw as f32 * 1.9073486328125e-4 - 50.0; //  ==> / 1048576.0 * 200 - 50;
        Ok(Reading { temp, hum })
    }

    async fn reset(&mut self) -> Result<(), E> {
        let status = self.read_status().await?;
        if status & 0x18 != 0x18 {
            // TODO: logs
            // info!("resetting");
            self.write_data(&[0x1B, 0, 0]).await?;
            self.write_data(&[0x1C, 0, 0]).await?;
            self.write_data(&[0x1E, 0, 0]).await?;
        }
        Ok(())
    }

    async fn read_data(&mut self) -> Result<[u8; 8], E> {
        let mut buffer = [0; 8];
        self.i2c.read(self.address, &mut buffer).await?;
        Ok(buffer)
    }

    async fn read_status(&mut self) -> Result<u8, E> {
        let mut buffer = [0; 1];
        self.i2c.read(self.address, &mut buffer).await?;
        Ok(buffer[0])
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<(), E> {
        self.i2c.write(self.address, data).await
    }
}
