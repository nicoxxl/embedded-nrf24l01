use crate::command::{ReadRxPayload, ReadRxPayloadWidth};
use crate::config::Configuration;
use crate::device::Device;
use crate::payload::Payload;
use crate::registers::{FifoStatus, CD};
use crate::standby::StandbyMode;
use core::fmt;

pub struct RxMode<D: Device> {
    device: D,
}

impl<D: Device> fmt::Debug for RxMode<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RxMode")
    }
}

impl<D: Device> RxMode<D> {
    /// Relies on everything being set up by `StandbyMode::rx()`, from
    /// which it is called
    pub(crate) fn new(device: D) -> Self {
        RxMode { device }
    }

    /// Disable `CE` so that you can switch into TX mode.
    pub fn standby(self) -> StandbyMode<D> {
        StandbyMode::from_rx_tx(self.device)
    }

    /// Is there any incoming data to read? Return the pipe number.
    pub fn can_read(&mut self) -> Result<Option<u8>, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(status, fifo_status)| {
                if !fifo_status.rx_empty() {
                    Some(status.rx_p_no())
                } else {
                    None
                }
            })
    }

    /// Is an in-band RF signal detected?
    ///
    /// The internal carrier detect signal must be high for 40μs
    /// (NRF24L01+) or 128μs (NRF24L01) before the carrier detect
    /// register is set. Note that changing from standby to receive
    /// mode also takes 130μs.
    pub fn has_carrier(&mut self) -> Result<bool, D::Error> {
        self.device
            .read_register::<CD>()
            .map(|(_, cd)| cd.0 & 1 == 1)
    }

    /// Is the RX queue empty?
    pub fn is_empty(&mut self) -> Result<bool, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(_, fifo_status)| fifo_status.rx_empty())
    }

    /// Is the RX queue full?
    pub fn is_full(&mut self) -> Result<bool, D::Error> {
        self.device
            .read_register::<FifoStatus>()
            .map(|(_, fifo_status)| fifo_status.rx_full())
    }

    pub fn read(&mut self) -> Result<Payload, D::Error> {
        let (_, payload_width) = self.device.send_command(&ReadRxPayloadWidth)?;
        let (_, payload) = self
            .device
            .send_command(&ReadRxPayload::new(payload_width as usize))?;
        Ok(payload)
    }
}

impl<D: Device> Configuration for RxMode<D> {
    type Inner = D;
    fn device(&mut self) -> &mut Self::Inner {
        &mut self.device
    }
}
