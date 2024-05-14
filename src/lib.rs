use ggrs::*;
use std::collections::BTreeMap;

// Types
type CppSessionHandle = u32;
type CppPlayerHandle = u32;

// Consts
pub const INVALID_HANDLE: CppSessionHandle = 0;

pub struct GGRSCPPConfig;
impl ggrs::Config for GGRSCPPConfig
{
    type Input = u32;
    type State = u32;
    type Address = u32;
}

#[derive(Clone, Copy)]
pub struct SessionBuilderSettings{
    max_prediction: usize,
    fps: usize
}
impl SessionBuilderSettings {
    const fn new() -> Self {
        Self{
            max_prediction: 8,
            fps: 60
        }
    }
}

#[repr(u8)]
enum GgrsCppRequestType{
    AdvanceFrame,
    SetInput,
    LoadGameState,
    SaveGameState,
    None
}

#[repr(C)]
pub struct GgrsCppRequest {
    request_type: GgrsCppRequestType,
    frame: Frame,
    player_handle: CppPlayerHandle,
    input: u32
}

enum SessionType {
    SyncTest
}

enum Session{
    SyncTest(SyncTestSession<GGRSCPPConfig>)
}

static mut SB_SETTINGS: SessionBuilderSettings = SessionBuilderSettings::new();
static mut SESSIONS: BTreeMap<CppSessionHandle, Session> = BTreeMap::new();

//////////////////////////////
// SessionBuilder Functions //
//////////////////////////////
#[no_mangle]
pub extern fn ggrs_sessionBuilder__new() {
    unsafe{
        SB_SETTINGS = SessionBuilderSettings::new();
    }
}

#[no_mangle]
pub extern fn ggrs_sessionBuilder__with_fps(fps: usize) {
    unsafe{
        SB_SETTINGS.fps = fps;
    }
}

fn build_session(session_type: SessionType) -> Result<CppSessionHandle, GgrsError> {
    static mut SESSION_HANDLE: CppSessionHandle = 1;

    let handle: CppSessionHandle;
    let settings: SessionBuilderSettings;

    unsafe{
        settings = SB_SETTINGS;
        handle = SESSION_HANDLE;
        SESSION_HANDLE += 1;
    }

    let sb = SessionBuilder::<GGRSCPPConfig>::new()
        .with_fps(settings.fps)?
        .with_max_prediction_window(settings.max_prediction)?;


    match session_type {
        SessionType::SyncTest => {
            let sess = sb.start_synctest_session()?;
            unsafe{
                SESSIONS.insert(handle, Session::SyncTest(sess));
            }
        }
    }

    Ok(handle)
}

#[no_mangle]
pub extern fn ggrs_sessionBuilder__start_synctest_session() -> CppSessionHandle{
    match build_session(SessionType::SyncTest) {
        Ok(h) => h,
        Err(_) => INVALID_HANDLE
    }
}

#[no_mangle]
pub extern fn ggrs_session__advance_frame(handle: CppSessionHandle) {
    let requests: Vec<GgrsRequest<GGRSCPPConfig>>;

    // Get requests
    unsafe {
        match SESSIONS.get_mut(&handle) {
            Some(sess) => {
                match sess {
                    Session::SyncTest(st) => {
                        match st.advance_frame() {
                            Ok(req) => {
                                requests = req
                            }
                            Err(_) => return
                        }
                    }
                };
            }
            None => return
        };
    }

    // Convert requests to GgrsCppRequest's
}

#[no_mangle]
pub extern fn ggrs_session__next_ggrsRequest(handle: CppSessionHandle) -> GgrsCppRequest {
    GgrsCppRequest {
        request_type: GgrsCppRequestType::None,
        frame: 0,
        player_handle: 0,
        input: 0
    }
}

