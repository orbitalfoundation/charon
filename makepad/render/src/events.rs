use crate::cx::*;
use makepad_microserde::*;
use std::any::TypeId;
use std::collections::{HashMap,BTreeSet};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub logo: bool
}

#[derive(Clone, Debug, PartialEq)]
pub enum FingerInputType{
    Mouse,
    Touch,
    XR
}

impl FingerInputType{
    pub fn is_touch(&self)->bool{*self == FingerInputType::Touch}
    pub fn is_mouse(&self)->bool{*self == FingerInputType::Mouse}
    pub fn is_xr(&self)->bool{*self == FingerInputType::XR}
    pub fn has_hovers(&self)->bool{ *self == FingerInputType::Mouse || *self == FingerInputType::XR}
}

impl Default for FingerInputType{
    fn default()->Self{Self::Mouse}
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerDownEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub digit: usize,
    pub tap_count: u32,
    pub handled: bool,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerMoveEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub is_over: bool,
    pub digit: usize,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64
}

impl FingerMoveEvent {
    pub fn move_distance(&self) -> f32 {
        ((self.abs_start.x - self.abs.x).powf(2.) + (self.abs_start.y - self.abs.y).powf(2.)).sqrt()
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerUpEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub abs_start: Vec2,
    pub rel: Vec2,
    pub rel_start: Vec2,
    pub rect: Rect,
    pub digit: usize,
    pub is_over: bool,
    pub input_type: FingerInputType,
    pub modifiers: KeyModifiers,
    pub time: f64
}

#[derive(Clone, Debug, PartialEq)]
pub enum HoverState {
    In,
    Over,
    Out
}

impl Default for HoverState {
    fn default() -> HoverState {
        HoverState::Over
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerHoverEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub any_down: bool,
    pub handled: bool,
    pub hover_state: HoverState,
    pub modifiers: KeyModifiers,
    pub time: f64
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FingerScrollEvent {
    pub window_id: usize,
    pub digit: usize,
    pub abs: Vec2,
    pub rel: Vec2,
    pub rect: Rect,
    pub scroll: Vec2,
    pub input_type: FingerInputType,
    //pub is_wheel: bool,
    pub handled_x: bool,
    pub handled_y: bool,
    pub modifiers: KeyModifiers,
    pub time: f64
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowGeomChangeEvent {
    pub window_id: usize,
    pub old_geom: WindowGeom,
    pub new_geom: WindowGeom,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowMovedEvent {
    pub window_id: usize,
    pub old_pos: Vec2,
    pub new_pos: Vec2,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct AnimateEvent {
    pub frame: u64,
    pub time: f64
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct NextFrameEvent {
    pub frame: u64,
    pub time: f64
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileReadEvent {
    pub read_id: u64,
    pub data: Result<Vec<u8>, String>
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimerEvent {
    pub timer_id: u64
}

#[derive(Clone, Debug, PartialEq)]
pub struct SignalEvent {
    pub signals: HashMap<Signal, BTreeSet<StatusId>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct TriggersEvent {
    pub triggers: HashMap<Area, BTreeSet<TriggerId>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct TriggerEvent {
    pub triggers: BTreeSet<TriggerId>
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileWriteEvent {
    id: u64,
    error: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveRecompileEvent {
    pub changed_live_bodies: BTreeSet<LiveBodyId>,
    pub errors: Vec<LiveBodyError>
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    //pub key_char: char,
    pub is_repeat: bool,
    pub modifiers: KeyModifiers,
    pub time: f64
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyFocusEvent {
    pub prev: Area,
    pub focus: Area,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputEvent {
    pub input: String,
    pub replace_last: bool,
    pub was_paste: bool
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextCopyEvent {
    pub response: Option<String>
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowCloseRequestedEvent {
    pub window_id: usize,
    pub accept_close: bool
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowClosedEvent {
    pub window_id: usize
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowResizeLoopEvent {
    pub was_started: bool,
    pub window_id: usize
}

#[derive(Clone, Debug, PartialEq)]
pub enum WindowDragQueryResponse {
    NoAnswer,
    Client,
    Caption,
    SysMenu, // windows only
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowDragQueryEvent {
    pub window_id: usize,
    pub abs: Vec2,
    pub response: WindowDragQueryResponse,
}

#[derive(Clone, Debug, Default, SerBin, DeBin, PartialEq)]
pub struct XRButton {
    pub value:f32,
    pub pressed:bool
}

#[derive(Clone, Debug, Default, SerBin, DeBin,PartialEq)]
pub struct XRInput {
    pub active: bool,
    pub grip: Transform,
    pub ray: Transform,
    pub num_buttons: usize,
    pub buttons: [XRButton;8],
    pub num_axes: usize,
    pub axes: [f32;8],
}

#[derive(Clone, Debug, SerBin, DeBin, PartialEq)]
pub struct XRUpdateEvent {
    // alright what data are we stuffing in 
    pub time: f64,
    pub head_transform: Transform,
    pub left_input: XRInput,
    pub last_left_input: XRInput,
    pub right_input: XRInput,
    pub last_right_input: XRInput,
    pub other_inputs: Vec<XRInput>
}

#[derive(Clone, Debug, PartialEq)]
pub struct WebSocketMessageEvent{
    pub url: String, 
    pub result: Result<Vec<u8>, String>
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    None,
    Construct,
    Destruct,
    Draw,
    Paint,
    AppFocus,
    AppFocusLost,
    AnimEnded(AnimateEvent),
    Animate(AnimateEvent),
    NextFrame(NextFrameEvent),
    XRUpdate(XRUpdateEvent),
    WindowSetHoverCursor(MouseCursor),
    WindowDragQuery(WindowDragQueryEvent),
    WindowCloseRequested(WindowCloseRequestedEvent),
    WindowClosed(WindowClosedEvent),
    WindowGeomChange(WindowGeomChangeEvent),
    WindowResizeLoop(WindowResizeLoopEvent),
    FingerDown(FingerDownEvent),
    FingerMove(FingerMoveEvent),
    FingerHover(FingerHoverEvent),
    FingerUp(FingerUpEvent),
    FingerScroll(FingerScrollEvent),
    FileRead(FileReadEvent),
    FileWrite(FileWriteEvent),
    Timer(TimerEvent),
    Signal(SignalEvent),
    Triggers(TriggersEvent),
    Trigger(TriggerEvent),
    Command(CommandId),
    KeyFocus(KeyFocusEvent),
    KeyFocusLost(KeyFocusEvent),
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
    TextInput(TextInputEvent),
    TextCopy(TextCopyEvent),
    LiveRecompile(LiveRecompileEvent),
    WebSocketMessage(WebSocketMessageEvent),
}

impl Default for Event {
    fn default() -> Event {
        Event::None
    }
}

pub enum HitTouch {
    Single,
    Multi
}

#[derive(Clone, Debug, Default)]
pub struct HitOpt {
    pub use_multi_touch: bool,
    pub margin: Option<Margin>,
}

impl Event {
    
    pub fn is_next_frame(&self, cx:&mut Cx, next_frame: NextFrame)->Option<NextFrameEvent>{
         match self {
            Event::NextFrame(fe) => {
                if cx._next_frames.contains(&next_frame){
                   return Some(fe.clone()) 
                }
            }
            _=>()
        }
        None
    }

    pub fn is_animate(&self, cx:&mut Cx, animator: &Animator)->Option<AnimateEvent>{
         match self {
            Event::Animate(ae) => {
                if cx.playing_animator_ids.get(&animator.animator_id).is_some(){
                    return Some(ae.clone())
                }
            }
            _=>()
        }
        None
    }
    
    pub fn hits(&mut self, cx: &mut Cx, area: Area, opt: HitOpt) -> Event {
        match self {
            Event::KeyFocus(kf) => {
                if area == kf.prev {
                    return Event::KeyFocusLost(kf.clone())
                }
                else if area == kf.focus {
                    return Event::KeyFocus(kf.clone())
                }
            },
            Event::KeyDown(_) => {
                if area == cx.key_focus {
                    return self.clone();
                }
            },
            Event::KeyUp(_) => {
                if area == cx.key_focus {
                    return self.clone();
                }
            },
            Event::TextInput(_) => {
                if area == cx.key_focus {
                    return self.clone();
                }
            },
            Event::TextCopy(_) => {
                if area == cx.key_focus {
                    return Event::TextCopy(
                        TextCopyEvent {response: None}
                    );
                }
            },
            Event::Triggers(te) => {
                if let Some(triggers) = te.triggers.get(&area).cloned(){
                    return Event::Trigger(TriggerEvent{triggers})
                }
            }
            Event::FingerScroll(fe) => {
                let rect = area.get_rect(&cx);
                if rect.contains_with_margin(fe.abs, &opt.margin) {
                    //fe.handled = true;
                    return Event::FingerScroll(FingerScrollEvent {
                        rel: fe.abs - rect.pos,
                        rect: rect,
                        ..fe.clone()
                    })
                }
            },
            Event::FingerHover(fe) => {
                let rect = area.get_rect(&cx);
                
                if cx.fingers[fe.digit]._over_last == area {
                    let mut any_down = false;
                    for finger in &cx.fingers {
                        if finger.captured == area {
                            any_down = true;
                            break;
                        }
                    }
                    if !fe.handled && rect.contains_with_margin(fe.abs, &opt.margin) {
                        fe.handled = true;
                        if let HoverState::Out = fe.hover_state {
                            //    cx.finger_over_last_area = Area::Empty;
                        }
                        else {
                            cx.fingers[fe.digit].over_last = area;
                        }
                        return Event::FingerHover(FingerHoverEvent {
                            rel: area.abs_to_rel(cx, fe.abs),
                            rect: rect,
                            any_down:any_down,
                            ..fe.clone()
                        })
                    }
                    else {
                        //self.was_over_last_call = false;
                        return Event::FingerHover(FingerHoverEvent {
                            rel: area.abs_to_rel(cx, fe.abs),
                            rect: rect,
                            any_down:any_down,
                            hover_state: HoverState::Out,
                            ..fe.clone()
                        })
                    }
                }
                else {
                    if !fe.handled && rect.contains_with_margin(fe.abs, &opt.margin) {
                        let mut any_down = false;
                        for finger in &cx.fingers {
                            if finger.captured == area {
                                any_down = true;
                                break;
                            }
                        }
                        cx.fingers[fe.digit].over_last = area;
                        fe.handled = true;
                        //self.was_over_last_call = true;
                        return Event::FingerHover(FingerHoverEvent {
                            rel: area.abs_to_rel(cx, fe.abs),
                            rect: rect,
                            any_down:any_down,
                            hover_state: HoverState::In,
                            ..fe.clone()
                        })
                    }
                }
            },
            Event::FingerMove(fe) => {
                // check wether our digit is captured, otherwise don't send
                if cx.fingers[fe.digit].captured == area {
                    let abs_start = cx.fingers[fe.digit].down_abs_start;
                    let rel_start = cx.fingers[fe.digit].down_rel_start;
                    let rect = area.get_rect(&cx);
                    return Event::FingerMove(FingerMoveEvent {
                        abs_start: abs_start,
                        rel: area.abs_to_rel(cx, fe.abs),
                        rel_start: rel_start,
                        rect: rect,
                        is_over: rect.contains_with_margin(fe.abs, &opt.margin),
                        ..fe.clone()
                    })
                }
            },
            Event::FingerDown(fe) => {
                if !fe.handled {
                    let rect = area.get_rect(&cx);
                    if rect.contains_with_margin(fe.abs, &opt.margin) {
                        // scan if any of the fingers already captured this area
                        if !opt.use_multi_touch {
                            for finger in &cx.fingers {
                                if finger.captured == area {
                                    return Event::None;
                                }
                            }
                        }
                        cx.fingers[fe.digit].captured = area;
                        let rel = area.abs_to_rel(cx, fe.abs);
                        cx.fingers[fe.digit].down_abs_start = fe.abs;
                        cx.fingers[fe.digit].down_rel_start = rel;
                        fe.handled = true;
                        return Event::FingerDown(FingerDownEvent {
                            rel: rel,
                            rect: rect,
                            ..fe.clone()
                        })
                    }
                }
            },
            Event::FingerUp(fe) => {
                if cx.fingers[fe.digit].captured == area {
                    cx.fingers[fe.digit].captured = Area::Empty;
                    let abs_start = cx.fingers[fe.digit].down_abs_start;
                    let rel_start = cx.fingers[fe.digit].down_rel_start;
                    let rect = area.get_rect(&cx);
                    return Event::FingerUp(FingerUpEvent {
                        is_over: rect.contains(fe.abs),
                        abs_start: abs_start,
                        rel_start: rel_start,
                        rel: area.abs_to_rel(cx, fe.abs),
                        rect: rect,
                        ..fe.clone()
                    })
                }
            },
            _ => ()
        };
        return Event::None;
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, Default)]
pub struct Signal {
    pub signal_id: usize
}

impl Signal {
    pub fn empty() -> Signal {
        Signal {
            signal_id: 0
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.signal_id == 0
    }
}


// Status


#[derive(PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Eq, Debug)]
pub struct StatusId(pub TypeId);

impl Into<StatusId> for TypeId {
    fn into(self) -> StatusId {StatusId(self)}
}

#[derive(PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Eq, Debug)]
pub struct TriggerId(pub TypeId);

impl Into<TriggerId> for TypeId {
    fn into(self) -> TriggerId {TriggerId(self)}
}




#[derive(Clone, Debug, Default)]
pub struct FileRead {
    pub path: String,
    pub read_id: u64
}

impl FileRead {
    pub fn is_pending(&self) -> bool {
        self.read_id != 0
    }
    
    pub fn resolve_utf8<'a>(&mut self, fr: &'a FileReadEvent) -> Option<Result<&'a str,String>> {
        if fr.read_id == self.read_id {
            self.read_id = 0;
            if let Ok(str_data) = &fr.data {
                if let Ok(utf8_string) = std::str::from_utf8(&str_data) {
                    return Some(Ok(utf8_string))
                }
                else {
                    return Some(Err(format!("can't parse file as utf8 {}", self.path)))
                }
            }
            else if let Err(err) = &fr.data {
                return Some(Err(format!("can't load file as utf8 {} {}", self.path, err)))
            }
        }
        return None
    }
}

#[derive(Clone, Debug, Default)]
pub struct Timer {
    pub timer_id: u64
}

impl Timer {
    pub fn empty() -> Timer {
        Timer {
            timer_id: 0,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.timer_id == 0
    }
    
    pub fn is_timer(&mut self, te: &TimerEvent) -> bool {
        te.timer_id == self.timer_id
    }
}

impl Event {
    pub fn set_handled(&mut self, set: bool) {
        match self {
            Event::FingerHover(fe) => {
                fe.handled = set;
            },
            Event::FingerDown(fe) => {
                fe.handled = set;
            },
            _ => ()
        }
    }
    
    pub fn handled(&self) -> bool {
        match self {
            Event::FingerHover(fe) => {
                fe.handled
            },
            Event::FingerDown(fe) => {
                fe.handled
            },
            
            _ => false
        }
    }
    
    
}

// lowest common denominator keymap between desktop and web
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum KeyCode {
    Escape,
    
    Backtick,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Minus,
    Equals,
    
    Backspace,
    Tab,
    
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LBracket,
    RBracket,
    Return,
    
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Backslash,
    
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,
    
    Control,
    Alt,
    Shift,
    Logo,
    
    //RightControl,
    //RightShift,
    //RightAlt,
    //RightLogo,
    
    Space,
    Capslock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    
    PrintScreen,
    Scrolllock,
    Pause,
    
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    
    NumpadEquals,
    NumpadSubtract,
    NumpadAdd,
    NumpadDecimal,
    NumpadMultiply,
    NumpadDivide,
    Numlock,
    NumpadEnter,
    
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    
    Unknown
}

impl Default for KeyCode{
    fn default()->Self{KeyCode::Unknown}
}