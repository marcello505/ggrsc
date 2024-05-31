use ggrs::*;
use std::collections::{BTreeMap, VecDeque};
use std::vec::Vec;

#[cfg(not(feature = "c_socket"))]
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
#[cfg(not(feature = "c_socket"))]
use std::ffi::{CStr, c_char};

// Modules
#[cfg(feature = "c_socket")]
mod socket;

// Types
pub type CSessionHandle = u32;
pub type CPlayerHandle = usize;
pub type CFrame = i32;
pub type CInput = u32;

// Consts
pub const INVALID_HANDLE: CSessionHandle = 0;

pub struct CConfig;
#[cfg(not(feature = "c_socket"))]
impl ggrs::Config for CConfig
{
    type Input = CInput;
    type State = u32;
    type Address = SocketAddr;
}
#[cfg(feature = "c_socket")]
impl ggrs::Config for CConfig
{
    type Input = CInput;
    type State = u32;
    type Address = socket::CAddressHandle;
}


//#[derive(Clone, Copy)]
pub struct CSessionBuilderSettings{
    max_prediction: usize,
    fps: usize,
    num_players: usize,
    sparse_saving: bool,
    local_player_handles: Vec<CPlayerHandle>,
#[cfg(not(feature = "c_socket"))]
    remote_player_handles: Vec<(CPlayerHandle, SocketAddr)>,
#[cfg(feature = "c_socket")]
    remote_player_handles: Vec<(CPlayerHandle, socket::CAddressHandle)>,
    host_port: u16,
    input_delay: usize
}
impl CSessionBuilderSettings {
    const fn new() -> Self {
        Self{
            max_prediction: 8,
            fps: 60,
            num_players: 2,
            sparse_saving: false,
            local_player_handles: Vec::new(),
            remote_player_handles: Vec::new(),
            host_port: 30000,
            input_delay: 2
        }
    }
}

#[repr(u8)]
pub enum CRequestType{
    AdvanceFrame,
    SetInput,
    LoadGameState,
    SaveGameState,
    None
}

#[repr(u8)]
pub enum CSessionState {
    Synchronizing,
    Running
}

#[repr(C)]
pub struct CRequest {
    request_type: CRequestType,
    frame: CFrame,
    player_handle: CPlayerHandle,
    input: CInput
}
impl CRequest {
    const fn new_none() -> Self {
        Self {
            request_type: CRequestType::None,
            frame: 0,
            player_handle: 0,
            input: 0
        }
    }

    const fn new_save(frame: CFrame) -> Self {
        Self {
            request_type: CRequestType::SaveGameState,
            frame,
            player_handle: 0,
            input: 0
        }
    }

    const fn new_load(frame: CFrame) -> Self {
        Self {
            request_type: CRequestType::LoadGameState,
            frame,
            player_handle: 0,
            input: 0
        }
    }

    const fn new_advance() -> Self {
        Self {
            request_type: CRequestType::AdvanceFrame,
            frame: 0,
            player_handle: 0,
            input: 0
        }
    }

    const fn new_input(player_handle: CPlayerHandle, input: CInput) -> Self {
        Self {
            request_type: CRequestType::SetInput,
            frame: 0,
            player_handle,
            input
        }
    }
}

enum CSessionType {
    SyncTest,
    P2P
}

enum CSession{
    SyncTest(SyncTestSession<CConfig>),
    P2P(P2PSession<CConfig>)
}

static mut SB_SETTINGS: CSessionBuilderSettings = CSessionBuilderSettings::new();
static mut SESSIONS: BTreeMap<CSessionHandle, CSession> = BTreeMap::new();
static mut REQUESTS: BTreeMap<CSessionHandle, VecDeque<CRequest>> = BTreeMap::new();

//////////////////////////////
// SessionBuilder Functions //
//////////////////////////////
#[no_mangle]
pub extern fn ggrs_builder_new() {
    unsafe{
        SB_SETTINGS = CSessionBuilderSettings::new();
    }
}

#[no_mangle]
pub extern fn ggrs_builder_with_fps(fps: usize) {
    unsafe{
        SB_SETTINGS.fps = fps;
    }
}

#[no_mangle]
pub extern fn ggrs_builder_with_max_prediction_window(window: usize) {
    unsafe{
        SB_SETTINGS.max_prediction = window;
    }
}

#[no_mangle]
pub extern fn ggrs_builder_with_num_players(num_players: usize) {
    unsafe{
        SB_SETTINGS.num_players = num_players;
    }
}

#[no_mangle]
pub extern fn ggrs_builder_with_sparse_saving_mode(sparse_saving: bool) {
    unsafe{
        SB_SETTINGS.sparse_saving = sparse_saving;
    }
}

#[no_mangle]
pub extern fn ggrs_builder_with_input_delay(delay: usize) {
    unsafe{
        SB_SETTINGS.input_delay = delay;
    }
}

#[no_mangle]
pub extern fn ggrs_builder_add_local_player(player_handle: CPlayerHandle) {
    unsafe{
        SB_SETTINGS.local_player_handles.push(player_handle);
    }
}

#[cfg(not(feature = "c_socket"))]
#[no_mangle]
pub extern fn ggrs_builder_add_remote_player_ipv4(player_handle: CPlayerHandle, ipv4: *const c_char, port: u16) {
    unsafe{
        let ipv4_cstr = CStr::from_ptr(ipv4);
        let addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(ipv4_cstr.to_str().unwrap().parse().unwrap(), port));
        SB_SETTINGS.remote_player_handles.push((player_handle, addr));
    }
}

#[cfg(not(feature = "c_socket"))]
#[no_mangle]
pub extern fn ggrs_builder_add_remote_player_ipv6(player_handle: CPlayerHandle, ipv6: *const c_char, port: u16) {
    unsafe{
        let ipv6_cstr = CStr::from_ptr(ipv6);
        let addr: SocketAddr = SocketAddr::V6(SocketAddrV6::new(ipv6_cstr.to_str().unwrap().parse().unwrap(), port, 0, 0));
        SB_SETTINGS.remote_player_handles.push((player_handle, addr));
    }
}

#[cfg(feature = "c_socket")]
#[no_mangle]
pub extern fn ggrs_builder_add_remote_player(player_handle: CPlayerHandle, addr_handle: socket::CAddressHandle) {
    unsafe{
        SB_SETTINGS.remote_player_handles.push((player_handle, addr_handle));
    }
}

#[no_mangle]
pub extern fn ggrs_builder_set_host_port(port: u16) {
    unsafe{
        SB_SETTINGS.host_port = port;
    }
}

fn build_session(session_type: CSessionType) -> Result<CSessionHandle, GgrsError> {
    static mut SESSION_HANDLE: CSessionHandle = 1;

    let handle: CSessionHandle;
    let mut sb: SessionBuilder<CConfig>;

    unsafe{
        handle = SESSION_HANDLE;
        SESSION_HANDLE += 1;
        sb = SessionBuilder::<CConfig>::new()
            .with_fps(SB_SETTINGS.fps)?
            .with_max_prediction_window(SB_SETTINGS.max_prediction)?
            .with_num_players(SB_SETTINGS.num_players)
            .with_sparse_saving_mode(SB_SETTINGS.sparse_saving)
            .with_input_delay(SB_SETTINGS.input_delay);

        // Add local player handles
        for i in 0..SB_SETTINGS.local_player_handles.len() {
            sb = sb.add_player(PlayerType::Local, SB_SETTINGS.local_player_handles[i])?;
        }

        // Add remote player handles
        for i in 0..SB_SETTINGS.remote_player_handles.len() {
            sb = sb.add_player(PlayerType::Remote(SB_SETTINGS.remote_player_handles[i].1), SB_SETTINGS.remote_player_handles[i].0)?;
        }
    }

    match session_type {
        CSessionType::SyncTest => {
            let sess = sb.start_synctest_session()?;
            unsafe{
                SESSIONS.insert(handle, CSession::SyncTest(sess));
                REQUESTS.insert(handle, VecDeque::new());
            }
        }
        CSessionType::P2P => {
            unsafe{
                #[cfg(not(feature = "c_socket"))]
                let sess = sb.start_p2p_session(UdpNonBlockingSocket::bind_to_port(SB_SETTINGS.host_port).unwrap())?;
                #[cfg(feature = "c_socket")]
                let sess = sb.start_p2p_session(socket::CSocket::new(handle))?;
                SESSIONS.insert(handle, CSession::P2P(sess));
                REQUESTS.insert(handle, VecDeque::new());
                #[cfg(feature = "c_socket")]
                socket::SOCKET_IN.insert(handle, VecDeque::new());
                #[cfg(feature = "c_socket")]
                socket::SOCKET_OUT.insert(handle, VecDeque::new());
            }
        }
    }

    Ok(handle)
}

#[no_mangle]
pub extern fn ggrs_builder_start_synctest_session() -> CSessionHandle{
    match build_session(CSessionType::SyncTest) {
        Ok(h) => h,
        Err(_) => INVALID_HANDLE
    }
}

#[no_mangle]
pub extern fn ggrs_builder_start_p2p_session() -> CSessionHandle{
    match build_session(CSessionType::P2P) {
        Ok(h) => h,
        Err(_) => INVALID_HANDLE
    }
}

#[no_mangle]
pub extern fn ggrs_session_poll_remote_clients(handle: CSessionHandle)
{
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    CSession::SyncTest(_) => {
                        return;
                    }
                    CSession::P2P(p2p) => {
                        p2p.poll_remote_clients();
                    }
                };
            }
            None => return
        };
    }
}

#[no_mangle]
pub extern fn ggrs_session_current_state(handle: CSessionHandle) -> CSessionState
{
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    CSession::SyncTest(_) => {
                        CSessionState::Running
                    }
                    CSession::P2P(p2p) => {
                        match p2p.current_state() {
                            SessionState::Synchronizing => CSessionState::Synchronizing,
                            SessionState::Running => CSessionState::Running
                        }
                    }
                }
            }
            None => CSessionState::Running
        }
    }
}

#[no_mangle]
pub extern fn ggrs_session_frames_ahead(handle: CSessionHandle) -> i32
{
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    CSession::SyncTest(_) => {
                        0
                    }
                    CSession::P2P(p2p) => {
                        p2p.frames_ahead()
                    }
                }
            }
            None => 0
        }
    }
}

#[no_mangle]
pub extern fn ggrs_session_add_local_input(handle: CSessionHandle, player_handle: CPlayerHandle, input: CInput) {
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    CSession::SyncTest(st) => {
                        st.add_local_input(player_handle, input).unwrap();
                    }
                    CSession::P2P(p2p) => {
                        p2p.add_local_input(player_handle, input).unwrap();
                    }
                };
            }
            None => return
        };
    }
}

#[no_mangle]
pub extern fn ggrs_session_advance_frame(handle: CSessionHandle) {
    let ggrs_requests: Vec<GgrsRequest<CConfig>>;
    let c_requests: &mut VecDeque<CRequest>;

    // Get ggrs_requests and c_requests
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    CSession::SyncTest(st) => {
                        match st.advance_frame() {
                            Ok(req) => {
                                ggrs_requests = req
                            }
                            Err(_) => return
                        }
                    }
                    CSession::P2P(p2p) => {
                        match p2p.advance_frame() {
                            Ok(req) => {
                                ggrs_requests = req
                            }
                            Err(_) => return
                        }
                    }
                };
            }
            None => return
        };

        match REQUESTS.get_mut(&handle) {
            Some(reqs) => {
                c_requests = reqs;
            }
            None => return
        }
    }

    // Convert requests to GgrsCppRequest's
    for req in ggrs_requests {
        match req {
            GgrsRequest::SaveGameState{ frame, cell } => {
                cell.save(frame, None, None);
                c_requests.push_back(CRequest::new_save(frame));
            }

            GgrsRequest::LoadGameState { frame, cell } => {
                cell.load();
                c_requests.push_back(CRequest::new_load(frame));
            }

            GgrsRequest::AdvanceFrame { inputs } => {
                for i in 0..inputs.len() {
                    c_requests.push_back(CRequest::new_input(i, inputs[i].0));
                }
                
                c_requests.push_back(CRequest::new_advance());
            }
        }

    }

}

#[no_mangle]
pub extern fn ggrs_session_next_ggrsRequest(handle: CSessionHandle) -> CRequest {
    unsafe {
        match REQUESTS.get_mut(&handle) {
            Some(reqs) => {
                match reqs.pop_front() {
                    Some(r) => return r,
                    None => return CRequest::new_none()
                };
            }

            None => return CRequest::new_none()
        };
    }
}

