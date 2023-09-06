#[derive(Debug, PartialEq)]
enum AvrNetMode {
    AddressMsb,
    AddressLsb,
    LengthMsb,
    LengthLsb,
    Data,
}

pub(crate) struct AvrNetState {
    my_address: u16,
    message_address: u16,
    length: u16,
    index: usize,
    data: [u8; 256],
    mode: AvrNetMode,
}

#[derive(Debug)]
pub(crate) struct AvrNetMessage {
    pub(crate) address: u16,
    pub(crate) data: Vec<u8>,
}

impl TryFrom<AvrNetMessage> for Vec<u8> {
    type Error = &'static str;

    fn try_from(msg: AvrNetMessage) -> Result<Self, Self::Error> {
        if msg.data.len() > u16::MAX as usize {
            return Err("Invalid message length");
        }

        let mut data = Vec::new();

        // Address
        data.push((msg.address >> 8) as u8);
        data.push(msg.address as u8);

        // Length
        let len: u16 = msg.data.len() as u16;
        data.push((len >> 8) as u8);
        data.push(len as u8);

        // Data
        data.extend(msg.data.iter());

        Ok(data)
    }
}

impl TryFrom<Vec<u8>> for AvrNetMessage {
    type Error = &'static str;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        if (data.len() < 4) || (data.len() > (u16::MAX as usize + 4)) {
            return Err("Invalid message length");
        }

        let mut msg = AvrNetMessage {
            address: 0,
            data: Vec::new(),
        };

        // Address
        msg.address = (data[0] as u16) << 8;
        msg.address |= data[1] as u16;

        // Length
        let len: u16 = ((data[2] as u16) << 8) | (data[3] as u16);

        // Data
        msg.data = data[4..4 + len as usize].to_vec();

        if msg.data.len() != len as usize {
            return Err("Invalid message length");
        }

        Ok(msg)
    }
}

impl AvrNetState {
    pub(crate) fn new(my_address: u16) -> AvrNetState {
        AvrNetState {
            my_address,
            message_address: 0,
            length: 0,
            index: 0,
            data: [0; 256],
            mode: AvrNetMode::AddressMsb,
        }
    }

    fn reset(&mut self) {
        self.message_address = 0;
        self.length = 0;
        self.index = 0;
        self.mode = AvrNetMode::AddressMsb;
        self.data = [0; 256];
    }

    pub(crate) fn rx(&mut self, c: u8) -> Option<AvrNetMessage> {
        match self.mode {
            AvrNetMode::AddressMsb => {
                self.message_address = (c as u16) << 8;
                self.mode = AvrNetMode::AddressLsb;
            }
            AvrNetMode::AddressLsb => {
                self.message_address |= c as u16;
                self.mode = AvrNetMode::LengthMsb;
            }
            AvrNetMode::LengthMsb => {
                self.length = (c as u16) << 8;
                self.mode = AvrNetMode::LengthLsb;
            }
            AvrNetMode::LengthLsb => {
                self.length |= c as u16;
                self.mode = AvrNetMode::Data;
            }
            AvrNetMode::Data => {
                self.data[self.index] = c;
                self.index += 1;

                if self.index == self.length as usize {
                    self.mode = AvrNetMode::AddressMsb;
                    if self.message_address == self.my_address {
                        let data = self.data[0..self.length as usize].to_vec();
                        let message = AvrNetMessage {
                            address: self.message_address,
                            data,
                        };
                        self.reset();
                        return Some(message);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::avr_net::AvrNetMessage;

    #[test]
    fn message_from_vec() {
        let data = vec![0, 1, 0, 2, 4, 2];
        let message = AvrNetMessage::try_from(data).unwrap();

        assert_eq!(message.address, 1);
        assert_eq!(message.data, vec![4, 2]);
    }

    #[test]
    fn message_to_vec() {
        let message = AvrNetMessage {
            address: 1,
            data: vec![4, 2],
        };

        let data: Vec<u8> = message.try_into().unwrap();
        assert_eq!(data, vec![0, 1, 0, 2, 4, 2]);
    }
}
