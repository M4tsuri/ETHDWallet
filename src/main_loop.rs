use cortex_m::interrupt::free;
use stm32f4xx_hal::gpio::ExtiPin;
use alloc::vec::Vec;

use crate::{
    global::*, 
    update_global, 
    wallet::{
        initializer::try_initialize_wallet, 
        Wallet, safe_zone::{Signature, EthAddr}, ACCOUNT_NUM, utils::get_cipher
    }, 
    input::{
        MsgBufferState, 
        MsgBuffer, KeyInputBuffer
    }, 
    error::{self, Error}
};

pub fn main_loop() -> ! {
    update_global!(|mut keyboard: Option<KEY_TRIGGER>, mut exti: Option<EXTI>| {
        keyboard.enable_interrupt(&mut exti);
    });

    let wallet = try_initialize_wallet();

    loop {
        cortex_m::asm::wfi();
        update_global!(|mut buf: Copy<MSG_BUFFER>| {
            let result = match buf.state {
                MsgBufferState::Finished => {
                    dispatch(buf, wallet);
                },
                MsgBufferState::Invalid => {
                    Error::SerialDataCorrupted;
                },
                _ => {}
            };


            // this will become the new global buffer
            buf = MsgBuffer::new()
        });
    }
}

fn error_report() {
    todo!()
}

/// defines an instruction
#[repr(u8)]
enum Instruction<'raw> {
    /// [0, account_id, raw]
    SignTransaction(u8, &'raw [u8]),
    /// [1, account_id]
    GetAddress(u8),
    /// [2]
    GetAddressList
}

#[repr(u8)]
enum Response {
    Signature(Signature),
    Address(EthAddr),
    AddressList([EthAddr; ACCOUNT_NUM])
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        match self {
            Response::Signature(Signature { 
                r, s, v 
            }) => {
                let mut res = Vec::new();
                res.push(0);
                res.extend_from_slice(&r);
                res.extend_from_slice(&s);
                res.push(v);
                res
            },
            Response::Address(addr) => {
                addr.to_vec()
            },
            Response::AddressList(list) => {
                list.into_iter().flatten().collect()
            },
        }
    }
}

impl<'raw> TryFrom<&'raw [u8]> for Instruction<'raw> {
    type Error = error::Error;

    fn try_from(value: &'raw [u8]) -> Result<Self, Self::Error> {
        Ok(match *value.get(0).ok_or(Error::InvalidInstruction)? {
            0 if value.len() > 2 => {
                Self::SignTransaction(value[1], &value[2..])
            },
            1 if value.len() == 2 => Self::GetAddress(value[1]),
            2 if value.len() == 1 => Self::GetAddressList,
            _ => return Err(Error::InvalidInstruction)
        })
    }
}

fn dispatch(buf: MsgBuffer, wallet: &mut Wallet) -> error::Result<Response> {
    let instr: Instruction = (&buf.buf[..buf.msg_len as usize]).try_into()?;

    Ok(match instr {
        Instruction::SignTransaction(idx, raw) => {
            let cipher = wallet.cipher.get_or_insert(get_cipher(
                KeyInputBuffer::wait_for_key(), &wallet.chacha_iv
            ));
            
            Response::Signature(
                wallet.zone.sign_raw(raw, idx as usize, cipher)?
            )
        },
        Instruction::GetAddress(idx) => {
            Response::Address(
                *wallet.addrs.get(idx as usize)
                    .ok_or(Error::AccountIdxOOB)?
            )
        },
        Instruction::GetAddressList => {
            Response::AddressList(wallet.addrs)
        },
    })
}
