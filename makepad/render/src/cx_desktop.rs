use crate::cx::*;
use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::net::TcpStream;
//use time::precise_time_ns;

#[macro_export]
macro_rules!log {
    ( $ ( $t: tt) *) => {
        println!("{}:{} - {}",file!(),line!(),format!($($t)*))
    }
}

#[derive(Clone)]
pub struct CxDesktop {
    pub repaint_via_scroll_event: bool,
    pub file_read_id: u64,
    pub file_reads: Vec<FileRead>,
    pub profiler_start: Option<u64>,
}

impl Default for CxDesktop {
    fn default() -> CxDesktop {
        CxDesktop {
            repaint_via_scroll_event: false,
            file_read_id: 1,
            file_reads: Vec::new(),
            profiler_start: None,
        }
    }
}

impl Cx {
    
    pub fn get_default_window_size(&self) -> Vec2 {
        return Vec2 {x: 800., y: 600.}
    }
    
    pub fn file_read(&mut self, path: &str) -> FileRead {
        let desktop = &mut self.platform.desktop;
        desktop.file_read_id += 1;
        let read_id = desktop.file_read_id;
        let file_read = FileRead {
            read_id: read_id,
            path: path.to_string()
        };
        desktop.file_reads.push(file_read.clone());
        file_read
    }
    
    pub fn file_write(&mut self, path: &str, data: &[u8]) -> u64 {
        // just write it right now
        if let Ok(mut file) = File::create(path) {
            if let Ok(_) = file.write_all(&data) {
            }
            else {
                println!("ERROR WRITING FILE {}", path);
            }
        }
        else {
            println!("ERROR WRITING FILE {}", path);
        }
        0
    }
    
    pub fn process_desktop_pre_event(&mut self, event: &mut Event)
    {
        match event {
            Event::FingerHover(fe) => {
                self.fingers[fe.digit].over_last = Area::Empty;
                //self.hover_mouse_cursor = None;
            },
            Event::FingerUp(_fe) => {
                self.down_mouse_cursor = None;
            },
            Event::WindowCloseRequested(_cr) => {
            },
            Event::FingerDown(fe) => {
                // lets set the finger tap count
                fe.tap_count = self.process_tap_count(fe.digit, fe.abs, fe.time);
            },
            Event::KeyDown(ke) => {
                self.process_key_down(ke.clone());
                if ke.key_code == KeyCode::PrintScreen {
                    if ke.modifiers.control {
                        self.panic_redraw = true;
                    }
                    else {
                        self.panic_now = true;
                    }
                }
            },
            Event::KeyUp(ke) => {
                self.process_key_up(&ke);
            },
            Event::AppFocusLost => {
                self.call_all_keys_up();
            },
            _ => ()
        };
    }
    
    pub fn process_desktop_post_event(&mut self, event: &mut Event) -> bool {
        match event {
            Event::FingerUp(fe) => { // decapture automatically
                self.fingers[fe.digit].captured = Area::Empty;
            },
            Event::FingerHover(fe) => { // new last area finger over
                self.fingers[fe.digit]._over_last = self.fingers[fe.digit].over_last;
                //if fe.hover_state == HoverState::Out{
                //    self.hover_mouse_cursor = None;
                //}
            },
            Event::FingerScroll(_) => {
                // check for anything being paint or dra dirty
                if self.redraw_child_areas.len()>0 || self.redraw_parent_areas.len()>0 {
                    self.platform.desktop.repaint_via_scroll_event = true;
                }
            }
            _ => {}
        }
        false
    }
    
    pub fn process_desktop_paint_callbacks(&mut self, time: f64) -> bool
    {
        if self.playing_animator_ids.len() != 0 {
            self.call_animate_event(time);
        }
        
        let mut vsync = false; //self.platform.desktop.repaint_via_scroll_event;
        self.platform.desktop.repaint_via_scroll_event = false;
        if self.next_frames.len() != 0 {
            self.call_next_frame_event(time);
            if self.next_frames.len() != 0 {
                vsync = true;
            }
        }
        
        self.call_signals_and_triggers();
        
        // call redraw event
        if self.redraw_child_areas.len()>0 || self.redraw_parent_areas.len()>0 {
            self.call_draw_event();
        }
        if self.redraw_child_areas.len()>0 || self.redraw_parent_areas.len()>0 {
            vsync = true;
        }
        
        self.process_desktop_file_reads();
        
        self.call_signals_and_triggers();
        
        vsync
    }
    
    
    pub fn process_desktop_file_reads(&mut self){
        if self.platform.desktop.file_reads.len() == 0 {
            return
        }
        
        let file_read_requests = self.platform.desktop.file_reads.clone();
        self.platform.desktop.file_reads.truncate(0);
        
        for read_req in file_read_requests {
            let file_result = File::open(&read_req.path);
            if let Ok(mut file) = file_result {
                let mut buffer = Vec::new();
                // read the whole file
                if file.read_to_end(&mut buffer).is_ok() {
                    self.call_event_handler(&mut Event::FileRead(FileReadEvent {
                        read_id: read_req.read_id,
                        data: Ok(buffer)
                    }))
                }
                else {
                    self.call_event_handler(&mut Event::FileRead(FileReadEvent {
                        read_id: read_req.read_id,
                        data: Err(format!("Failed to read {}", read_req.path))
                    }))
                }
            }
            else {
                self.call_event_handler(&mut Event::FileRead(FileReadEvent {
                    read_id: read_req.read_id,
                    data: Err(format!("Failed to open {}", read_req.path))
                }))
            }
        }
        
        if self.platform.desktop.file_reads.len() != 0 {
            self.process_desktop_file_reads();
        }
    }
    
    pub fn process_to_wasm<F>(&mut self, _msg: u32, mut _event_handler: F) -> u32
    where F: FnMut(&mut Cx, &mut Event)
    {
        0
    }
    
    pub fn load_all_fonts(&mut self) {
        
        self.fonts.resize(self.live_styles.font_index.len(), CxFont::default());
        // lets load all fonts that aren't loaded yet
        for (file, font) in &self.live_styles.font_index {
            let file = file.to_string();
            let cxfont = &mut self.fonts[font.font_id];
            if let Ok(mut file_handle) = File::open(&file) {
                let mut buffer = Vec::<u8>::new();
                if file_handle.read_to_end(&mut buffer).is_ok() {
                    if cxfont.load_from_ttf_bytes(&buffer).is_err() {
                        println!("Error loading font {} ", file);
                    }
                    else {
                        cxfont.file = file;
                    }
                }
            }
            else {
                println!("Error loading font {} ", file);
            }
        }
    }
    
    /*pub fn log(&mut self, val:&str){
        let mut stdout = io::stdout();
        let _e = stdout.write(val.as_bytes());
        let _e = stdout.flush();
    }*/
    
    pub fn write_log(data: &str) {
        let _ = io::stdout().write(data.as_bytes());
        let _ = io::stdout().flush();
    }
    
    pub fn websocket_send(&self, _url: &str, _data: &[u8]) {
        // nop
    }
    
    pub fn http_send(&self, verb: &str, path: &str, _proto: &str, domain: &str, port: u16, content_type: &str, body: &[u8], signal: Signal) {
        
        fn write_bytes_to_tcp_stream(tcp_stream: &mut TcpStream, bytes: &[u8]) -> bool {
            let bytes_total = bytes.len();
            let mut bytes_left = bytes_total;
            while bytes_left > 0 {
                let buf = &bytes[(bytes_total - bytes_left)..bytes_total];
                if let Ok(bytes_written) = tcp_stream.write(buf) {
                    if bytes_written == 0 {
                        return false
                    }
                    bytes_left -= bytes_written;
                }
                else {
                    return true
                }
            }
            return false
        }
        
        // start a thread, connect, and report back.
        let data = body.to_vec();
        let byte_len = data.len();
        let header = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nConnect: close\r\nContent-Type:{}\r\nContent-Length:{}\r\n\r\n",
            verb,
            path,
            domain,
            content_type,
            byte_len
        );
        let host = format!("{}:{}", domain, port);
        let _connect_thread = {
            std::thread::spawn(move || {
                let stream = TcpStream::connect(&host);
                if let Ok(mut stream) = stream {
                    if !write_bytes_to_tcp_stream(&mut stream, header.as_bytes())
                        && !write_bytes_to_tcp_stream(&mut stream, &data) {
                        Cx::post_signal(signal, Cx::status_http_send_ok());
                        return
                    }
                }
                Cx::post_signal(signal, Cx::status_http_send_fail());
            })
        };
    }
    /*
    
    pub fn profile(&mut self) {
        if let Some(start) = self.platform.desktop.profiler_start {
            let delta = mach_absolute_time() - start;
            println!("Profile time:{} usec", delta / 1_000);
            self.platform.desktop.profiler_start = None
        }
        else {
            self.platform.desktop.profiler_start = Some(precise_time_ns())
        }
    }*/
}
