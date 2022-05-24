use cortex_m::prelude::*;
use stm32f4::stm32f407::USART1;
use stm32f4xx_hal::{serial::Tx, block};
use crate::error::Result;

use crate::{
    global::*, 
    update_global, 
    wallet::{
        Wallet, wallet,
        safe_zone::{Signature, EthAddr},
        ACCOUNT_NUM
    }, 
    input::{
        MsgBufferState, 
        MsgBuffer, KeyInputBuffer
    }, 
    error::{self, Error}
};

pub fn main_loop() -> ! {
    let wallet = wallet();

    loop {
        cortex_m::asm::wfi();
        let _result: Result<()> = update_global!(|
            mut buf: Copy<MSG_BUFFER>, 
            mut tx: Option<SERIAL_TX>
        | {
            match match buf.state {
                MsgBufferState::Finished => {
                    dispatch(buf, wallet)
                },
                MsgBufferState::Invalid => {
                    Err(Error::SerialDataCorrupted)
                },
                _ => return Ok(())
            } {
                Ok(resp) => {
                    block!(tx.write(0x00))?;
                    resp.write_tx(&mut tx)?;
                },
                Err(e) => {
                    block!(tx.write(0xff))?;
                    block!(tx.write(e as u8))?;
                },
            }

            tx.bflush()?;
            // this will become the new global buffer
            buf = MsgBuffer::new();
            Ok(())
        });
    }
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

impl Response {
    fn write_tx(&self, tx: &mut Tx<USART1>) -> Result<()> {
        match self {
            Response::Signature(Signature { 
                r, s, v 
            }) => {
                block!(tx.write(0x00))?;
                tx.bwrite_all(r)?;
                tx.bwrite_all(s)?;
                block!(tx.write(*v))?;
            },
            Response::Address(addr) => {
                block!(tx.write(0x01))?;
                tx.bwrite_all(addr)?;
            },
            Response::AddressList(list) => {
                block!(tx.write(0x02))?;
                list.into_iter().try_for_each(|addr| {
                    tx.bwrite_all(addr)
                })?;
            },
        }
        Ok(())
    }
}

impl<'raw> TryFrom<&'raw [u8]> for Instruction<'raw> {
    type Error = error::Error;

    fn try_from(value: &'raw [u8]) -> core::result::Result<Self, Self::Error> {
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

fn dispatch(buf: MsgBuffer, wallet: &Wallet) -> error::Result<Response> {
    let instr: Instruction = (&buf.buf[..buf.msg_len as usize]).try_into()?;

    Ok(match instr {
        Instruction::SignTransaction(idx, raw) => {
            wallet.fill_cipher(KeyInputBuffer::wait_for_key())?;
            
            Response::Signature(
                wallet.sign_raw(idx as usize, raw)?
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
