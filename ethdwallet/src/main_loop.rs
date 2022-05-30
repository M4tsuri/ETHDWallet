use core::sync::atomic::Ordering;

use cortex_m::interrupt::free;
use cortex_m::prelude::*;
use stm32f4::stm32f407::USART1;
use stm32f4xx_hal::{serial::Tx, block};
use crate::error::Result;

use crate::wallet::{wallet, PubKey};
use crate::{
    global::*, 
    update_global, 
    wallet::{
        Wallet,
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
    loop {
        cortex_m::asm::wfi();
        let buf = update_global!(|buf: Copy<MSG_BUFFER>| {
            buf
        });

        let result = match buf.state {
            MsgBufferState::Finished => {
                dispatch(buf, wallet())
            },
            MsgBufferState::Error(e) => {
                Err(e)
            }
            _ => continue
        };

        let _result: Result<()> = update_global!(|
            mut buf: Copy<MSG_BUFFER>, 
            mut tx: Option<SERIAL_TX>
        | {
            match result  {
                Ok(resp) => {
                    // feed dog
                    WATCHDOG.store(true, Ordering::Relaxed);
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
    Address((EthAddr, PubKey)),
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
            Response::Address((addr, pubkey)) => {
                block!(tx.write(0x01))?;
                tx.bwrite_all(addr)?;
                tx.bwrite_all(pubkey)?;
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
    if !wallet.initialized {
        return Err(Error::WalletNotInitialized)
    }
    
    let instr: Instruction = (&buf.buf[..buf.msg_len as usize]).try_into()?;

    Ok(match instr {
        Instruction::SignTransaction(idx, raw) => {
            if free(|cs| {
                let cipher = CIPHER.borrow(cs).take();
                let is_none = cipher.is_none();
                CIPHER.borrow(cs).set(cipher);
                is_none
            }) {
                wallet.fill_cipher(KeyInputBuffer::wait_for_key())?;
            }
           
            Response::Signature(
                wallet.sign_raw(idx as usize, raw)?
            )
        },
        Instruction::GetAddress(idx) => {
            if idx as usize >= ACCOUNT_NUM {
                return Err(Error::AccountIdxOOB)
            }
            Response::Address((
                wallet.addrs[idx as usize],
                wallet.pubkeys[idx as usize]
            ))
        },
        Instruction::GetAddressList => {
            Response::AddressList(wallet.addrs)
        },
    })
}
