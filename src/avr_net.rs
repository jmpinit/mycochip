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

    pub(crate) fn rx(&mut self, c: u8) -> Option<Vec<u8>> {
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
                        self.reset();
                        return Some(data);
                    }
                }
            }
        }

        None
    }
}
