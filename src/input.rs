pub const FIXED_KEY_LEN: usize = 8;

pub enum KeyInputState {
    Reading(usize),
    Finished
}

pub struct KeyInputBuffer {
    pub buf: [u8; FIXED_KEY_LEN],
    pub state: KeyInputState
}

impl KeyInputBuffer {
    pub const fn new() -> Self {
        Self { 
            buf: [0; FIXED_KEY_LEN], 
            state: KeyInputState::Reading(0) 
        }
    }

    pub fn read(&mut self, byte: u8) {
        match self.state {
            KeyInputState::Reading(p) => {
                self.buf[p] = byte;
                // finished
                if p + 1 < FIXED_KEY_LEN {
                    self.state = KeyInputState::Reading(p + 1)
                } else {
                    self.state = KeyInputState::Finished
                }
            },
            KeyInputState::Finished => {},
        }
    }

    pub fn wait_for_key() -> [u8; 8] {
        todo!()
    }
}

pub const MSG_MAGIC: u8 = 0xff;

#[derive(Clone, Copy)]
pub enum MsgBufferState {
    PendingStart,
    PendingLen(u8),
    Reading(u32),
    Finished,
    Invalid,
}

pub const MAX_MSG_LEN: usize = 1024;

/// format of a message is MSG_MAGIC(1 byte) + MSG_LEN(4 byte) + MSG
#[derive(Clone, Copy)]
pub struct MsgBuffer {
    pub buf: [u8; MAX_MSG_LEN],
    pub msg_len: u32,
    pub state: MsgBufferState
}

impl MsgBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; MAX_MSG_LEN],
            msg_len: 0,
            state: MsgBufferState::PendingStart
        }
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
                    } else if self.msg_len >= MAX_MSG_LEN as u32 {
                        self.state = MsgBufferState::Invalid
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
            _ => {},
        }
    }
}
