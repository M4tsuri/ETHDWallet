#![feature(let_else)]

use std::{time::Duration, fmt::Display, str::FromStr};

use clap::{Parser, Subcommand};
use error::Error;
use num::BigUint;
use serialport::{self, SerialPort};
use tx::{UnsignedTx, SignedTx};
use web3::types::{H160, U256};
use rlp;

mod error;
mod tx;


pub const RINKEBY_ENDPOINT: &'static str = "https://rinkeby.infura.io/v3/2620729769024a63bf0c874a04fad486";
pub const RINKEBY_CHAINID: u32 = 4;

#[derive(Clone, Subcommand)]
#[clap(rename_all = "snake_case")]
pub enum Action {
    Sign {
        #[clap(short, long)]
        msg: String,
        #[clap(short, long)]
        account: u8
    },
    Transfer {
        #[clap(short, long)]
        to: String,
        #[clap(short, long)]
        value: String,
        #[clap(short, long)]
        account: u8
    },
    List,
    Get {
        #[clap(short, long)]
        account: u8
    }
}

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    serial: String,
    #[clap(short, long)]
    baudrate: u32,
    #[clap(subcommand)]
    action: Action
}

enum Instruction {
    /// [0, account_id, raw]
    SignTransaction(u8, Vec<u8>),
    /// [1, account_id]
    GetAddress(u8),
    /// [2]
    GetAddressList
}

impl Instruction {
    fn build_msg(self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.push(0xff);
    
        let mut content = Vec::new();
        match self {
            Instruction::SignTransaction(idx, raw) => {
                content.push(0x00);
                content.push(idx);
                content.extend(raw);
            },
            Instruction::GetAddress(idx) => {
                content.push(0x01);
                content.push(idx);
            },
            Instruction::GetAddressList => {
                content.push(0x02);
            },
        };

        msg.extend_from_slice(&(content.len() as u32).to_le_bytes());
        msg.extend(content);

        msg
    }
}

const ACCOUNT_NUM: usize = 32;
type EthAddr = [u8; 20];

#[derive(Clone, Copy, Debug)]
pub struct Signature {
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub v: u8
}

type PubKey = [u8; 64];

#[repr(u8)]
#[derive(Debug)]
enum Response {
    Signature(Signature),
    Address((EthAddr, PubKey)),
    AddressList([EthAddr; ACCOUNT_NUM])
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Signature(Signature { r, s, v }) => {
                write!(f, "r: {}, s: {}, v: {}", 
                    hex::encode(r),
                    hex::encode(s),
                    &v
                )
            },
            Response::Address((addr, pubkey)) => {
                write!(f, "address: 0x{}\n", hex::encode(addr))?;
                write!(f, "pubkey: 0x{}", hex::encode(pubkey))
            },
            Response::AddressList(addrs) => {
                addrs.iter().enumerate().try_for_each(|(idx, addr)| {
                    write!(f, "account {}: 0x{}\n", idx, hex::encode(addr))
                })
            },
        }
    }
}

fn process_instruction(
    serial: &mut dyn SerialPort, instr: Instruction
) -> Result<Response, error::Error> {

    let msg = instr.build_msg();
    // println!("send: {}", hex::encode(&msg));
    serial.write_all(&msg)?;

    let mut state = [0, 0];
    serial.read_exact(&mut state)?;

    match state[0] {
        0x00 => {},
        0xff => return Err(Error::WalletError(state[1].into())),
        _ => return Err(Error::SerialCorrupted)
    }

    Ok(match state[1] {
        0x00 => {
            let mut r = [0; 32];
            let mut s = [0; 32];
            let mut v = [0];

            serial.read_exact(&mut r)?;
            serial.read_exact(&mut s)?;
            serial.read_exact(&mut v)?;

            Response::Signature(Signature {
                r, s, v: v[0]
            })
        },
        0x01 => {
            let mut addr = [0; 20];
            let mut pubkey = [0; 64];
            serial.read_exact(&mut addr)?;
            serial.read_exact(&mut pubkey)?;

            Response::Address((addr, pubkey))
        },
        0x02 => {
            let mut addrs = [[0; 20]; ACCOUNT_NUM];
            addrs.iter_mut().try_for_each(|addr| {
                serial.read_exact(addr)?;
                Ok::<(), Error>(())
            })?;
            
            Response::AddressList(addrs)
        },
        _ => return Err(Error::SerialCorrupted)
    })
}

async fn process_action(
    serial: String, baudrate: u32, action: Action
) -> Result<(), error::Error> {
    let mut serial = serialport::new(serial, baudrate)
        .timeout(Duration::from_secs(1000))
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .open()?;

    let transport = web3::transports::Http::new(RINKEBY_ENDPOINT)?;
    let provider = web3::api::Web3::new(transport);


    Ok(match action {
        Action::Sign { msg, account } => {
            let msg = hex::decode(msg)?;
            let instr = Instruction::SignTransaction(account as u8, msg);
            let resp = process_instruction(serial.as_mut(), instr)?;

            println!("{}", resp)
        },
        Action::List => {
            let resp = process_instruction(
                serial.as_mut(), Instruction::GetAddressList
            )?;
            
            println!("{}", resp)
        },
        Action::Get { account } => {
            let resp = process_instruction(
                serial.as_mut(), Instruction::GetAddress(account)
            )?;

            println!("{}", resp)
        },
        Action::Transfer { to, value, account } => {
            let Response::Address((addr, _)) = process_instruction(
                serial.as_mut(), Instruction::GetAddress(account)
            )? else {
                panic!("type confusion.")
            };
            
            let from: H160 = addr.into();

            let mut nonce_buf = [0; 32];
            let nonce = provider.eth()
            .transaction_count(from, None).await?;
            nonce.to_big_endian(&mut nonce_buf);
            // println!("{}", nonce.trailing_zeros());

            let gasprice = provider.eth().gas_price().await?;

            let mut to_buf = [0; 20];
            let to_addr = hex::decode(to)?;
            if to_addr.len() != 20 {
                return Err(Error::ErrorAddressFormat);
            }
            to_buf.copy_from_slice(&to_addr);

            let unsigned_tx = UnsignedTx {
                nonce: nonce.as_u64(),
                gas_price: gasprice.as_u64(),
                gas_limit: 0x5208,
                to: to_buf,
                value: BigUint::from_str(&value)?,
                data: vec![],
                chainid: RINKEBY_CHAINID,
                _zero1: 0,
                _zero2: 0,
            };

            // (nonce, gasprice, startgas, to, value, data)
            let raw_unsigned = serlp::rlp::to_bytes(&unsigned_tx)?;
            

            let instr = Instruction::SignTransaction(
                account, raw_unsigned.to_vec()
            );

            let Response::Signature(sig) = process_instruction(serial.as_mut(), instr)? else {
                panic!("type confusion");
            };

            let signed_tx = unsigned_tx.into_signed(sig);

            let raw_signed = serlp::rlp::to_bytes(&signed_tx)?;

            println!(
                "unsigned: {}\nsigned: {}", 
                hex::encode(raw_unsigned.to_vec()),
                hex::encode(raw_signed.to_vec())
            );

            let receipt = provider.send_raw_transaction_with_confirmation(
                raw_signed.into(), 
                Duration::from_secs(1), 
                2
            ).await?;

            println!("{:?}", receipt);
        }
    })
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match process_action(args.serial, args.baudrate, args.action).await {
        Err(e) => println!("Error: {:?}", e),
        Ok(()) => {}
    }
}
