use crate::CSessionHandle;
use ggrs::Message;
use rmp_serde;

use std::collections::{BTreeMap, VecDeque};

pub const CMESSAGE_BUFFER_SIZE: usize = 255;

pub type CAddressHandle = u32;

#[repr(C)]
pub struct CMessage {
    addr: CAddressHandle,
    bytes: [u8; CMESSAGE_BUFFER_SIZE],
    bytes_length: u32
}

pub(crate) static mut SOCKET_OUT: BTreeMap<CSessionHandle, VecDeque<(CAddressHandle, Message)>> = BTreeMap::new();
pub(crate) static mut SOCKET_IN: BTreeMap<CSessionHandle, VecDeque<(CAddressHandle, Message)>> = BTreeMap::new();

pub struct CSocket {
    session_handle: CSessionHandle,
}
impl CSocket {
    pub const fn new(session_handle: CSessionHandle) -> Self {
        Self {
            session_handle
        }
    }
}

impl ggrs::NonBlockingSocket<CAddressHandle> for CSocket {
    fn send_to(&mut self, msg: &Message, addr: &CAddressHandle){
        unsafe {
            let sock_out = SOCKET_OUT.get_mut(&self.session_handle).unwrap();
            sock_out.push_back((*addr, msg.clone()));
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(CAddressHandle, Message)>{
        let mut result: Vec<(CAddressHandle, Message)> = Vec::new();
        
        unsafe {
            let sock_in = SOCKET_IN.get_mut(&self.session_handle).unwrap();
            while !sock_in.is_empty() {
                match sock_in.pop_front() {
                    Some(m) => {
                        result.push((m.0, m.1));
                    }
                    None => {}
                }
            }
        }
        result
    }
}

#[no_mangle]
pub extern fn ggrs_socket_in_message(session_handle: CSessionHandle, msg: &CMessage) {
    unsafe {
        let sock_in = SOCKET_IN.get_mut(&session_handle).unwrap();
        let msg_ggrs: Message = rmp_serde::from_slice(&msg.bytes[0..msg.bytes_length as usize]).unwrap();
        sock_in.push_back((msg.addr, msg_ggrs));
    }
}

#[no_mangle]
pub extern fn ggrs_socket_out_message(session_handle: CSessionHandle, msg: &mut CMessage) -> bool {
    unsafe {
        let sock_out = SOCKET_OUT.get_mut(&session_handle).unwrap();
        match sock_out.pop_front() {
            Some(m) => {
                let buf = rmp_serde::to_vec(&m.1).unwrap();

                msg.addr = m.0;
                msg.bytes_length = 0;
                for byte in buf {
                   msg.bytes[msg.bytes_length as usize] = byte;
                   msg.bytes_length += 1;
                }
                true
            },
            None => false
        }
    }
}
