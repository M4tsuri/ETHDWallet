pub struct KeyInputBuffer {
    cur: usize,
    buf: [u8; 8]
}

impl KeyInputBuffer {
    
}

pub const MSG_MAGIC: u8 = 0xff;

#[derive(Clone, Copy)]
pub enum MsgBufferState {
    PendingStart,
    PendingLen(u8),
    Reading(u32),
    Finished
}


/// format of a message is MSG_MAGIC(1 byte) + MSG_LEN(4 byte) + MSG
#[derive(Clone, Copy)]
pub struct MsgBuffer {
    pub buf: [u8; 1024],
    msg_len: u32,
    state: MsgBufferState
}

impl MsgBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; 1024],
            msg_len: 0,
            state: MsgBufferState::PendingStart
        }
    }

    pub fn state(&self) -> MsgBufferState {
        self.state
    }

    pub fn read(&mut self, byte: u8) {
        match self.state {
            MsgBufferState::PendingStart => {
                if byte == MSG_MAGIC {
                    self.state = MsgBufferState::PendingLen(0)
                }
            },
            MsgBufferState::PendingLen(p) => {
                self.msg_len |= (byte as u32) << ((p as u32) << 3);
                if p == 3 {
                    if self.msg_len == 0 { 
                        self.state = MsgBufferState::Finished 
                    } else {
                        self.state = MsgBufferState::Reading(0)
                    }                    
                } else {
                    self.state = MsgBufferState::PendingLen(p + 1)
                }
            },
            MsgBufferState::Reading(cur) => {
                self.buf[cur as usize] = byte;

                if cur + 1 == self.msg_len {
                    self.state = MsgBufferState::Finished
                } else {
                    self.state = MsgBufferState::Reading(cur + 1)
                }
            },
            MsgBufferState::Finished => {},
        }
    }
}
