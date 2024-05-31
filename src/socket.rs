use crate::CSessionHandle;
use ggrs::Message;
use rmp_serde;

use std::collections::{BTreeMap, VecDeque};

pub type CAddressHandle = u32;

#[repr(C)]
pub struct CMessage {
    valid: bool,
    addr: CAddressHandle,
    bytes: [u8; 255],
    bytes_length: u8
}
impl CMessage {
    fn new(addr: CAddressHandle) -> Self {
        Self {
            valid: true,
            addr,
            bytes: [0; 255],
            bytes_length: 0
        }
    }

    fn new_none() -> Self {
        Self {
            valid: false,
            addr: 0,
            bytes: [0; 255],
            bytes_length: 0
        }
    }
}

pub(crate) static mut SOCKET_OUT: BTreeMap<CSessionHandle, VecDeque<CMessage>> = BTreeMap::new();
pub(crate) static mut SOCKET_IN: BTreeMap<CSessionHandle, VecDeque<CMessage>> = BTreeMap::new();

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
        let mut c_msg: CMessage = CMessage::new(*addr);
        let buf = rmp_serde::to_vec(msg).unwrap();

        for byte in buf {
            c_msg.bytes[c_msg.bytes_length as usize] = byte;
            c_msg.bytes_length += 1;
        }
        
        unsafe {
            let sock_out = SOCKET_OUT.get_mut(&self.session_handle).unwrap();
            sock_out.push_back(c_msg);
        }
    }

    fn receive_all_messages(&mut self) -> Vec<(CAddressHandle, Message)>{
        let mut result: Vec<(CAddressHandle, Message)> = Vec::new();
        
        unsafe {
            let sock_in = SOCKET_IN.get_mut(&self.session_handle).unwrap();
            match sock_in.pop_front() {
                Some(cm) => {
                    let msg: Message = rmp_serde::from_slice(&cm.bytes[0..cm.bytes_length as usize]).unwrap();
                    result.push((cm.addr, msg))
                }
                None => {}
            }
        }

        result
    }
}

#[no_mangle]
pub extern fn ggrs_socket_in_message(session_handle: CSessionHandle, msg: CMessage) {
    unsafe {
        let sock_in = SOCKET_IN.get_mut(&session_handle).unwrap();
        sock_in.push_back(msg);
    }
}

#[no_mangle]
pub extern fn ggrs_socket_out_message(session_handle: CSessionHandle) -> CMessage {
    unsafe {
        let sock_out = SOCKET_OUT.get_mut(&session_handle).unwrap();
        match sock_out.pop_front() {
            Some(cm) => return cm,
            None => return CMessage::new_none()
        }
    }
}
