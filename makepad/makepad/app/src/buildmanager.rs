use makepad_render::*;
use makepad_widget::*;
use makepad_hub::*;
use crate::makepadstorage::*;
use crate::searchindex::*;

#[derive(Clone)]
pub struct BuildManager {
    pub signal: Signal,
    pub active_builds: Vec<ActiveBuild>,
    pub exec_when_done: bool,
    pub log_items: Vec<HubLogItem>,
    pub search_index: SearchIndex,
    pub tail_log_items: bool,
    pub artifacts: Vec<String>,
}

impl BuildManager {
    pub fn new(cx: &mut Cx) -> BuildManager {
        BuildManager {
            signal: cx.new_signal(),
            exec_when_done: false,
            log_items: Vec::new(),
            tail_log_items: true,
            artifacts: Vec::new(),
            active_builds: Vec::new(),
            search_index: SearchIndex::new(),
        }
    }
    
    pub fn status_new_log_item() -> StatusId {uid!()}
    pub fn status_new_artifact() -> StatusId {uid!()}
    pub fn status_cargo_end() -> StatusId {uid!()}
    pub fn status_program_end() -> StatusId {uid!()}
}

#[derive(Clone)]
pub struct ActiveBuild {
    pub build_target: BuildTarget,
    pub build_result: Option<BuildResult>,
    pub build_uid: Option<HubUid>,
    pub run_uid: Option<HubUid>,
}

impl BuildManager {
    
    fn clear_textbuffer_messages(&self, cx: &mut Cx, makepad_storage: &mut MakepadStorage) {
        // clear all files we missed
        for mtb in &mut makepad_storage.text_buffers {
            //if atb.text_buffer.messages.gc_id != cx.event_id {
            mtb.text_buffer.markers.message_cursors.truncate(0);
            mtb.text_buffer.markers.message_bodies.truncate(0);
            cx.send_signal(mtb.text_buffer.signal, TextBuffer::status_message_update());
            // }
            //else {
            //    cx.send_signal(atb.text_buffer.signal, SIGNAL_TEXTBUFFER_MESSAGE_UPDATE);
            //}
        }
    }
    
    pub fn is_running_uid(&self, uid: &HubUid) -> bool {
        for ab in &self.active_builds {
            if ab.build_uid == Some(*uid) {
                return true
            }
            if ab.run_uid == Some(*uid) {
                return true
            }
        }
        return false
    }
    
    pub fn is_any_cargo_running(&self) -> bool {
        for ab in &self.active_builds {
            if ab.build_uid.is_some() {
                return true
            }
        }
        return false
    }
    
    pub fn is_any_artifact_running(&self) -> bool {
        for ab in &self.active_builds {
            if ab.run_uid.is_some() {
                return true
            }
        }
        return false
    }
    
    pub fn process_loc_message_for_textbuffers(
        &self,
        cx: &mut Cx,
        loc_message: &LocMessage,
        level: TextBufferMessageLevel,
        makepad_storage: &mut MakepadStorage
    ) {
        let atb = makepad_storage.text_buffer_from_path(cx, &makepad_storage.remap_sync_path(&loc_message.path));
        let markers = &mut atb.text_buffer.markers;
        markers.mutation_id = atb.text_buffer.mutation_id.max(1);
        if markers.message_cursors.len() > 100000 { // crash saftey
            return
        }
        if let Some((head, tail)) = loc_message.range {
            let mut inserted = None;
            if markers.message_cursors.len()>0 {
                for i in (0..markers.message_cursors.len()).rev() {
                    if head >= markers.message_cursors[i].head {
                        break;
                    }
                    if head < markers.message_cursors[i].head && (i == 0 || head >= markers.message_cursors[i - 1].head) {
                        markers.message_cursors.insert(i, TextCursor {
                            head: head,
                            tail: tail,
                            max: 0
                        });
                        inserted = Some(i);
                        break;
                    }
                }
            }
            if inserted.is_none() {
                if let Some((head, tail)) = loc_message.range {
                    markers.message_cursors.push(TextCursor {
                        head: head,
                        tail: tail,
                        max: 0
                    })
                }
            }
            
            let msg = TextBufferMessage {
                body: loc_message.body.clone(),
                level: level
            };
            if let Some(pos) = inserted {
                atb.text_buffer.markers.message_bodies.insert(pos, msg);
            }
            else {
                atb.text_buffer.markers.message_bodies.push(msg);
            }
            cx.send_signal(atb.text_buffer.signal, TextBuffer::status_message_update());
        }
    }
    
    pub fn handle_log_item_limit(&mut self, cx: &mut Cx) {
        if self.log_items.len() >= 700000 { // out of memory safety
            if self.tail_log_items {
                self.log_items.truncate(500000);
                self.log_items.push(HubLogItem::Message("------------ Log truncated here -----------".to_string()));
            }
            else { // if not tailing, just throw it away
                if self.log_items.len() != 700001 {
                    self.log_items.push(HubLogItem::Message("------------ Log skipping, press tail to resume -----------".to_string()));
                    cx.send_signal(self.signal, BuildManager::status_new_log_item());
                }
                return
            }
        }
    }
    
    pub fn handle_hub_msg(
        &mut self,
        cx: &mut Cx,
        makepad_storage: &mut MakepadStorage,
        htc: &FromHubMsg
    ) {
        //let hub_ui = storage.hub_ui.as_mut().unwrap();
        match &htc.msg {
            HubMsg::ListBuildersResponse {..} => {
                self.restart_build(cx, makepad_storage);
            },
            HubMsg::CargoBegin {uid} => if self.is_running_uid(uid) {
                cx.send_signal(self.signal, BuildManager::status_new_log_item());
            },
            HubMsg::LogItem {uid, item} => if self.is_running_uid(uid) {
                
                self.handle_log_item_limit(cx);
                self.log_items.push(item.clone());
                if let Some(loc_message) = item.get_loc_message() {
                    let level = match item {
                        HubLogItem::LocPanic(_) => TextBufferMessageLevel::Log,
                        HubLogItem::LocError(_) => TextBufferMessageLevel::Error,
                        HubLogItem::LocWarning(_) => TextBufferMessageLevel::Warning,
                        HubLogItem::LocMessage(_) => TextBufferMessageLevel::Log,
                        HubLogItem::Error(_) => TextBufferMessageLevel::Error,
                        HubLogItem::Warning(_) => TextBufferMessageLevel::Warning,
                        HubLogItem::Message(_) => TextBufferMessageLevel::Log,
                    };
                    self.process_loc_message_for_textbuffers(cx, loc_message, level, makepad_storage)
                }
                cx.send_signal(self.signal, BuildManager::status_new_log_item());
            },
            
            HubMsg::CargoArtifact {uid, package_id, fresh: _} => if self.is_running_uid(uid) {
                self.artifacts.push(package_id.clone());
                cx.send_signal(self.signal, BuildManager::status_new_artifact());
            },
            HubMsg::BuildFailure {uid} => if self.is_running_uid(uid) {
                // if we didnt have any errors, check if we need to run
                for ab in &mut self.active_builds {
                    if ab.build_uid == Some(*uid) {
                        ab.build_uid = None;
                    }
                }
            },
            HubMsg::CargoEnd {uid, build_result} => if self.is_running_uid(uid) {
                for ab in &mut self.active_builds {
                    if ab.build_uid == Some(*uid) {
                        ab.build_uid = None;
                        ab.build_result = Some(build_result.clone());
                    }
                }
                if !self.is_any_cargo_running() && self.exec_when_done {
                    self.run_all_artifacts(makepad_storage)
                }
                cx.send_signal(self.signal, BuildManager::status_cargo_end());
            },
            HubMsg::ProgramEnd {uid} => if self.is_running_uid(uid) {
                // if we didnt have any errors, check if we need to run
                for ab in &mut self.active_builds {
                    if ab.run_uid == Some(*uid) {
                        ab.run_uid = None;
                    }
                }
                cx.send_signal(self.signal, BuildManager::status_program_end());
            },
            _ => ()
        }
    }
    
    pub fn add_log_message(&mut self, cx: &mut Cx, msg: String) {
        self.handle_log_item_limit(cx);
        self.log_items.push(HubLogItem::Message(msg));
        cx.send_signal(self.signal, BuildManager::status_new_log_item());
    }
    
    pub fn handle_live_recompile_event(
        &mut self,
        cx: &mut Cx,
        re: &LiveRecompileEvent,
        makepad_storage: &mut MakepadStorage
    ) {
        // we are running in loopback mode
        // lets use the log list as an error list for loopback shadercoding.
        // first of all
        // the problem is our path is not fully resolved
        // lets just map it to /main/makepad and worry later
        self.clear_textbuffer_messages(cx, makepad_storage);
        self.log_items.truncate(0);
        if re.errors.len() == 0 {
            self.log_items.push(HubLogItem::Message(
                format!("Live block compiled OK") 
            ));
        }
        for err in &re.errors {
            // lets turn line+col+len into a range.
            let path = format!("main/makepad/{}", err.file);
            // find the textbuffer
            let mtb = makepad_storage.text_buffer_from_path(cx, &path);
            // we should be able to mape line+col into byte offset
            let off = mtb.text_buffer.text_pos_to_offset(TextPos {row: err.line - 1, col: err.column - 1});
            let msg = LocMessage {
                path: path,
                line: err.line,
                column: err.column,
                body: err.message.clone(),
                range: Some((off, off + err.len)),
                rendered: None,
                explanation: None
            };
            self.process_loc_message_for_textbuffers(cx, &msg, TextBufferMessageLevel::Error, makepad_storage);
            self.log_items.push(HubLogItem::LocError(msg));
        }
        cx.send_signal(self.signal, BuildManager::status_new_log_item());
    }
    
    pub fn run_all_artifacts(&mut self, makepad_storage: &mut MakepadStorage) {
        let hub_ui = makepad_storage.hub_ui.as_mut().unwrap();
        // otherwise execute all we have artifacts for
        for ab in &mut self.active_builds {
            if let Some(build_result) = &ab.build_result {
                if let BuildResult::Executable {path} = build_result {
                    let uid = hub_ui.route_send.alloc_uid();
                    if let Some(run_uid) = ab.run_uid {
                        hub_ui.route_send.send(ToHubMsg {
                            to: HubMsgTo::Builder(ab.build_target.builder.clone()),
                            msg: HubMsg::ProgramKill {
                                uid: run_uid,
                            }
                        });
                    }
                    ab.run_uid = Some(uid);
                    hub_ui.route_send.send(ToHubMsg {
                        to: HubMsgTo::Builder(ab.build_target.builder.clone()),
                        msg: HubMsg::ProgramRun {
                            uid: ab.run_uid.unwrap(),
                            path: path.clone(),
                            args: Vec::new()
                        }
                    });
                }
            }
        }
    }
    
    pub fn artifact_run(&mut self, makepad_storage: &mut MakepadStorage) {
        if self.is_any_cargo_running() {
            self.exec_when_done = true;
        }
        else {
            self.run_all_artifacts(makepad_storage)
        }
    }
    
    pub fn restart_build(&mut self, cx: &mut Cx, makepad_storage: &mut MakepadStorage) {
        if !cx.platform_type.is_desktop() {
            return
        }
        
        self.artifacts.truncate(0);
        self.log_items.truncate(0);
        //self.selection.truncate(0);
        self.clear_textbuffer_messages(cx, makepad_storage);
        
        let hub_ui = makepad_storage.hub_ui.as_mut().unwrap();
        self.exec_when_done = makepad_storage.settings.exec_when_done;
        for ab in &mut self.active_builds {
            ab.build_result = None;
            if let Some(build_uid) = ab.build_uid {
                hub_ui.route_send.send(ToHubMsg {
                    to: HubMsgTo::Builder(ab.build_target.builder.clone()),
                    msg: HubMsg::BuildKill {
                        uid: build_uid,
                    }
                });
                ab.build_uid = None
            }
            if let Some(run_uid) = ab.run_uid {
                hub_ui.route_send.send(ToHubMsg {
                    to: HubMsgTo::Builder(ab.build_target.builder.clone()),
                    msg: HubMsg::ProgramKill {
                        uid: run_uid,
                    }
                });
                ab.run_uid = None
            }
        }
        
        // lets reset active targets
        self.active_builds.truncate(0);
        
        for build_target in &makepad_storage.settings.builds {
            let uid = hub_ui.route_send.alloc_uid();
            hub_ui.route_send.send(ToHubMsg {
                to: HubMsgTo::Builder(build_target.builder.clone()),
                msg: HubMsg::Build {
                    uid: uid.clone(),
                    workspace: build_target.workspace.clone(),
                    package: build_target.package.clone(),
                    config: build_target.config.clone()
                }
            });
            self.active_builds.push(ActiveBuild {
                build_target: build_target.clone(),
                build_result: None,
                build_uid: Some(uid),
                run_uid: None
            })
        }
    }
}
