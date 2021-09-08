use makepad_render::*;
use makepad_widget::*;


#[derive(Clone, DrawQuad)]
#[repr(C)]
pub struct DrawFileTreeFiller {
    #[default_shader(self::shader_filler)]
    base: DrawColor,
    line_vec: Vec2,
    anim_pos: f32
}


#[derive(Clone)]
pub struct FileTree {
    pub view: ScrollView,
    pub drag_view: View,
    pub _drag_move: Option<FingerMoveEvent>,
    pub root_node: FileNode,
    pub drag_bg: DrawColor,
    pub _shadow_area: Area,

    pub filler: DrawFileTreeFiller,
    pub tree_text: DrawText,
    pub node_bg: DrawColor,
    pub shadow: ScrollShadow,
    
    pub node_layout: Layout,
    pub row_height: f32,
    pub filler_walk: Walk,
    pub folder_walk: Walk,
    pub color_tree_folder: Vec4,
    pub color_tree_file: Vec4

}

#[derive(Clone, PartialEq)]
pub enum FileTreeEvent {
    None,
    DragMove {fe: FingerMoveEvent, paths: Vec<String>},
    DragCancel,
    DragEnd {fe: FingerUpEvent, paths: Vec<String>},
    DragOut,
    SelectFile {path: String},
    SelectFolder {path: String}
}

impl FileTree {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            root_node: FileNode::Folder {name: "".to_string(), state: NodeState::Open, draw: None, folder: vec![
                FileNode::File {name: "loading...".to_string(), draw: None},
            ]},

            drag_bg: DrawColor::new(cx, live_shader!(cx, self::shader_drag_bg)),

            view: ScrollView::new()
                .with_scroll_v(ScrollBar::new(cx).with_smoothing(0.15)),

            drag_view: View::new()
                .with_is_overlay(true),

            _drag_move: None,
            _shadow_area: Area::Empty,

            tree_text: DrawText::new(cx, default_shader!())
                .with_draw_depth(0.1),
            node_bg: DrawColor::new(cx, default_shader!()),
            //node_layout: LayoutFileTreeNode::id(),
            filler: DrawFileTreeFiller::new(cx, default_shader!())
                .with_draw_depth(0.2),

            shadow: ScrollShadow::new(cx)
                .with_draw_depth(0.25),

            node_layout: Layout::default(),
            row_height: 0.,
            filler_walk: Walk::default(),
            folder_walk: Walk::default(),
            color_tree_folder: Vec4::default(),
            color_tree_file: Vec4::default()
        }
    }
    
    pub fn style(cx: &mut Cx) {
        DrawFileTreeFiller::register_draw_input(cx);
        live_body!(cx, r#"
            self::shadow_size: 6.0;
            self::color_tree_folder: #f;
            self::color_tree_file: #9D9D9D;
            self::color_filler: #7f;
            self::color_bg_marked: #11466e;
            self::color_bg_selected: #28;
            self::color_bg_odd: #25;
            self::color_bg_marked_over: #11466e;
            self::color_bg_selected_over: #3d;
            self::color_bg_odd_over: #38;
            self::color_drag_bg: #11466e;
            
            self::layout_drag_bg: Layout {
                padding: {l: 5., t: 5., r: 5., b: 5.},
                walk: {
                    width: Compute,
                    height: Compute
                }
            }
            
            self::layout_node: Layout {
                walk: Walk {width: Fill, height: Fix(20.)},
                align: {fx: 0.0, fy: 0.5},
                padding: {l: 5., t: 0., r: 0., b: 1.},
            }
            
            self::text_style_label: TextStyle {
                top_drop: 1.3,
                ..makepad_widget::widgetstyle::text_style_normal
            }
            
            self::walk_filler: Walk {
                width: Fix(10),
                height: Fill,
                margin: {l: 1., t: 0., r: 4., b: 0.},
            }
            
            self::walk_folder: Walk {
                width: Fix(14.),
                height: Fill,
                margin: {l: 0., t: 0., r: 2., b: 0.}
            }
            
            self::shader_filler: Shader {
                use makepad_render::drawcolor::shader::*;
                
                draw_input: self::DrawFileTreeFiller;
                
                fn pixel() -> vec4 {
                    let df = Df::viewport(pos * rect_size);
                    let w = rect_size.x;
                    let h = rect_size.y;
                    if anim_pos< -0.5 {
                        df.move_to(0.5 * w, line_vec.x * h);
                        df.line_to(0.5 * w, line_vec.y * h);
                        return df.stroke(color * 0.5, 1.);
                    }
                    else { // its a folder
                        df.box(0. * w, 0.35 * h, 0.87 * w, 0.39 * h, 0.75);
                        df.box(0. * w, 0.28 * h, 0.5 * w, 0.3 * h, 1.);
                        df.union();
                        // ok so.
                        return df.fill(color);
                    }
                }
            }
            
            self::shader_drag_bg: Shader {
                use makepad_render::drawcolor::shader::*;
                
                fn pixel() -> vec4 {
                    let df = Df::viewport(pos * rect_size);
                    df.box(0., 0., rect_size.x, rect_size.y, 2.);
                    return df.fill(color);
                }
            }
            
            
        "#)
    }
    
    pub fn apply_style(&mut self, cx: &mut Cx) {
        self.filler.base.color = live_vec4!(cx, self::color_filler);
        self.node_layout = live_layout!(cx, self::layout_node);
        self.row_height = self.node_layout.walk.height.fixed();
        self.filler_walk = live_walk!(cx, self::walk_filler);
        self.folder_walk = live_walk!(cx, self::walk_folder);
        self.color_tree_folder = live_vec4!(cx, self::color_tree_folder);
        self.color_tree_file = live_vec4!(cx, self::color_tree_file);
        self.tree_text.text_style = live_text_style!(cx, self::text_style_label);
    }
    
    pub fn get_default_anim(cx: &Cx, counter: usize, marked: bool) -> Anim {
        let default_color = if marked {
            live_vec4!(cx, self::color_bg_marked)
        } else if counter & 1 == 0 {
            live_vec4!(cx, self::color_bg_selected)
        } else {
            live_vec4!(cx, self::color_bg_odd)
        };
        Anim {
            play: Play::Chain {duration: 0.01},
            tracks: vec![
                Track::Vec4 {
                    bind_to: live_item_id!(makepad_render::drawcolor::DrawColor::color),
                    ease: Ease::Lin,
                    keys: vec![(1.0, default_color)],
                    cut_init: None
                }
            ]
        }
    }
    
    pub fn get_over_anim(cx: &Cx, counter: usize, marked: bool) -> Anim {
        let over_color = if marked {
            live_vec4!(cx, self::color_bg_marked_over)
        } else if counter & 1 == 0 {
            live_vec4!(cx, self::color_bg_selected_over)
        } else {
            live_vec4!(cx, self::color_bg_odd_over)
        };
        Anim {
            play: Play::Cut {duration: 0.02},
            tracks: vec![
                Track::Vec4 {
                    bind_to: live_item_id!(makepad_render::drawcolor::DrawColor::color),
                    ease: Ease::Lin,
                    keys: vec![(0., over_color), (1., over_color)],
                    cut_init: None
                }
            ]
        }
    }
    
    pub fn clear_roots(&mut self, cx: &mut Cx, names: &Vec<String>) {
        self.root_node = FileNode::Folder {
            name: "".to_string(),
            draw: None,
            state: NodeState::Open,
            folder: names.iter().map( | v | FileNode::Folder {
                name: v.clone(),
                draw: None,
                state: NodeState::Open,
                folder: Vec::new()
            }).collect()
        };
        self.view.redraw_view(cx);
    }
    
    pub fn save_open_folders(&mut self) -> Vec<String> {
        let mut paths = Vec::new();
        fn recur_walk(node: &mut FileNode, base: &str, paths: &mut Vec<String>) {
            if node.is_open() {
                if let FileNode::Folder {folder, name, ..} = node {
                    let new_base = if name.len()>0 {
                        if base.len()>0 {format!("{}/{}", base, name)}else {name.to_string()}
                    }else {base.to_string()};
                    paths.push(new_base.clone());
                    for node in folder {
                        recur_walk(node, &new_base, paths)
                    }
                }
            }
        }
        recur_walk(&mut self.root_node, "", &mut paths);
        paths
    }
    
    pub fn load_open_folders(&mut self, cx: &mut Cx, paths: &Vec<String>) {
        
        fn recur_walk(node: &mut FileNode, base: &str, depth: usize, paths: &Vec<String>) {
            match node {
                FileNode::File {..} => (),
                FileNode::Folder {name, folder, state, ..} => {
                    let new_base = if name.len()>0 {
                        if base.len()>0 {format!("{}/{}", base, name)}else {name.to_string()}
                    }else {base.to_string()};
                    if depth == 0 || paths.iter().position( | v | *v == new_base).is_some() {
                        *state = NodeState::Open;
                        for node in folder {
                            recur_walk(node, &new_base, depth + 1, paths);
                        }
                    }
                    else {
                        *state = NodeState::Closed;
                    }
                }
            }
        }
        recur_walk(&mut self.root_node, "", 0, paths);
        self.view.redraw_view(cx);
    }
    
    
    pub fn get_marked_paths(root: &mut FileNode) -> Vec<String> {
        let mut paths = Vec::new();
        let mut file_walker = FileWalker::new(root);
        // make a path set of all marked items
        while let Some((_depth, _index, _len, node)) = file_walker.walk() {
            let node_draw = if let Some(node_draw) = node.get_draw() {node_draw}else {continue};
            if node_draw.marked != 0 {
                paths.push(file_walker.current_path());
            }
        }
        paths
    }
    
    pub fn handle_file_tree(&mut self, cx: &mut Cx, event: &mut Event) -> FileTreeEvent {
        
        // alright. someone clicking on the tree items.
        let mut file_walker = FileWalker::new(&mut self.root_node);
        let mut counter = 0;
        self.view.handle_scroll_view(cx, event);
        // todo, optimize this so events are not passed through 'all' of our tree elements
        // but filtered out somewhat based on a bounding rect
        let mut unmark_nodes = false;
        let mut drag_nodes = false;
        let mut drag_end: Option<FingerUpEvent> = None;
        let mut select_node = 0;
        while let Some((_depth, _index, _len, node)) = file_walker.walk() {
            // alright we haz a node. so now what.
            let is_filenode = if let FileNode::File {..} = node {true} else {false};
            
            let node_draw = if let Some(node_draw) = node.get_draw() {node_draw}else {continue};

            if let Some(ae) = event.is_animate(cx, &node_draw.animator) {
                self.node_bg.set_area(node_draw.area);
                self.node_bg.animate(cx, &mut node_draw.animator, ae.time);
                //node_draw.animator.handle_end(cx, ae.time);
            }
            
            match event.hits(cx, node_draw.area, HitOpt::default()) {
                Event::FingerDown(_fe) => {
                    // mark ourselves, unmark others
                    if is_filenode {
                        select_node = 1;
                    }
                    else {
                        select_node = 2;
                    }
                    node_draw.marked = cx.event_id;
                    
                    unmark_nodes = true;
                    node_draw.animator.play_anim(cx, Self::get_over_anim(cx, counter, node_draw.marked != 0));
                    
                    if let FileNode::Folder {state, ..} = node {
                        *state = match state {
                            NodeState::Opening(fac) => {
                                NodeState::Closing(1.0 - *fac)
                            },
                            NodeState::Closing(fac) => {
                                NodeState::Opening(1.0 - *fac)
                            },
                            NodeState::Open => {
                                NodeState::Closing(1.0)
                            },
                            NodeState::Closed => {
                                NodeState::Opening(1.0)
                            }
                        };
                        // start the redraw loop
                        self.view.redraw_view(cx);
                    }
                },
                Event::FingerUp(fe) => {
                    if !self._drag_move.is_none() {
                        drag_end = Some(fe);
                        // we now have to do the drop....
                        self.drag_view.redraw_view(cx);
                        //self._drag_move = None;
                    }
                },
                Event::FingerMove(fe) => {
                    cx.set_down_mouse_cursor(MouseCursor::Hand);
                    if self._drag_move.is_none() {
                        if fe.move_distance() > 50. {
                            self._drag_move = Some(fe);
                            self.view.redraw_view(cx);
                            self.drag_view.redraw_view(cx);
                        }
                    }
                    else {
                        self._drag_move = Some(fe);
                        self.view.redraw_view(cx);
                        self.drag_view.redraw_view(cx);
                    }
                    drag_nodes = true;
                },
                Event::FingerHover(fe) => {
                    cx.set_hover_mouse_cursor(MouseCursor::Hand);
                    match fe.hover_state {
                        HoverState::In => {
                            node_draw.animator.play_anim(cx, Self::get_over_anim(cx, counter, node_draw.marked != 0));
                        },
                        HoverState::Out => {
                            node_draw.animator.play_anim(cx, Self::get_default_anim(cx, counter, node_draw.marked != 0));
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
            counter += 1;
        }
        
        //unmark non selected nodes and also set even/odd animations to make sure its rendered properly
        if unmark_nodes {
            let mut file_walker = FileWalker::new(&mut self.root_node);
            let mut counter = 0;
            while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                if let Some(node_draw) = node.get_draw() {
                    if node_draw.marked != cx.event_id || node_draw.marked == 0 {
                        node_draw.marked = 0;
                        node_draw.animator.play_anim(cx, Self::get_default_anim(cx, counter, false));
                    }
                }
                if !file_walker.current_closing() {
                    counter += 1;
                }
            }
        }
        if let Some(fe) = drag_end {
            self._drag_move = None;
            let paths = Self::get_marked_paths(&mut self.root_node);
            if !self.view.area().get_rect(cx).contains(fe.abs) {
                return FileTreeEvent::DragEnd {
                    fe: fe.clone(),
                    paths: paths
                };
            }
        }
        if drag_nodes {
            if let Some(fe) = &self._drag_move {
                // lets check if we are over our own filetree
                // ifso, we need to support moving files with directories
                let paths = Self::get_marked_paths(&mut self.root_node);
                if !self.view.area().get_rect(cx).contains(fe.abs) {
                    return FileTreeEvent::DragMove {
                        fe: fe.clone(),
                        paths: paths
                    };
                }
                else {
                    return FileTreeEvent::DragCancel;
                }
            }
        };
        if select_node != 0 {
            let mut file_walker = FileWalker::new(&mut self.root_node);
            while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                let node_draw = if let Some(node_draw) = node.get_draw() {node_draw}else {continue};
                if node_draw.marked != 0 {
                    if select_node == 1 {
                        return FileTreeEvent::SelectFile {
                            path: file_walker.current_path()
                        };
                    }
                    else {
                        return FileTreeEvent::SelectFolder {
                            path: file_walker.current_path()
                        };
                    }
                }
            }
        }
        FileTreeEvent::None
    }
    
    pub fn draw_file_tree(&mut self, cx: &mut Cx) {
        if self.view.begin_view(cx, Layout::default()).is_err() {return}

        self.apply_style(cx);
        
        let mut file_walker = FileWalker::new(&mut self.root_node);
        
        // lets draw the filetree
        let mut counter = 0;
        let mut scale_stack = Vec::new();
        let mut last_stack = Vec::new();
        scale_stack.push(1.0f64);
        
        while let Some((depth, index, len, node)) = file_walker.walk() {
            
            let is_first = index == 0;
            let is_last = index == len - 1;
            
            while depth < scale_stack.len() {
                scale_stack.pop();
                last_stack.pop();
            }
            let scale = scale_stack[depth - 1];
            
            // lets store the bg area in the tree
            let node_draw = node.get_draw();
            if node_draw.is_none() {
                *node_draw = Some(NodeDraw {
                    area: Area::Empty,
                    animator: Animator::default(),
                    marked: 0
                })
            }
            let node_draw = node_draw.as_mut().unwrap();
            
            // using set area is necessary because we don't keep one instance
            // of the draw api with the item. 
            self.node_bg.set_area(node_draw.area);
            
             if node_draw.animator.need_init(cx) {
                node_draw.animator.init(cx, Self::get_default_anim(cx, counter, false));
            }
            // because we don't keep the draw api with the item
            // we have to pull the last values out of the animator
            self.node_bg.last_animate(&node_draw.animator);
            
            let mut node_layout = self.node_layout.clone();
            node_layout.walk.height = Height::Fix(self.row_height * scale as f32);

            self.node_bg.begin_quad(cx, node_layout);
            node_draw.area = self.node_bg.area();

            let is_marked = node_draw.marked != 0;
            
            for i in 0..(depth - 1) {
                if i == depth - 2 { // our own thread.
                    if is_last {
                        if is_first {
                            //line_vec
                            self.filler.line_vec = vec2(0.3, 0.7);
                        }
                        else {
                            //line_vec
                            self.filler.line_vec = vec2(-0.2,  0.7);
                        }
                    }
                    else if is_first {
                        //line_vec
                        self.filler.line_vec = vec2(-0.3, 1.2)
                    }
                    else {
                        //line_vec
                        self.filler.line_vec = vec2(-0.2, 1.2);
                    }
                    //anim_pos
                    self.filler.anim_pos = -1.;
                    self.filler.draw_quad_walk(cx, self.filler_walk);
                }
                else {
                    let here_last = if last_stack.len()>1 {last_stack[i + 1]} else {false};
                    if here_last {
                        cx.walk_turtle(self.filler_walk);
                    }
                    else {
                        self.filler.line_vec = vec2(-0.2, 1.2);
                        self.filler.anim_pos = -1.;
                        self.filler.draw_quad_walk(cx, self.filler_walk);
                    }
                }
            };
            //self.item_draw.filler.z = 0.;
            //self.item_draw.tree_text.z = 0.;
            //self.item_draw.tree_text.font_size = self.font_size;
            self.tree_text.font_scale = scale as f32;
            match node {
                FileNode::Folder {name, state, ..} => {
                    // draw the folder icon
                    self.filler.line_vec = vec2(0.,0.);
                    self.filler.anim_pos = 1.;
                    self.filler.draw_quad_walk(cx, self.folder_walk);
                    // move the turtle down a bit
                    //cx.move_turtle(0., 3.5);
                    cx.turtle_align_y();
                    //cx.realign_turtle(Align::left_center(), false);
                    self.tree_text.color = self.color_tree_folder;
                    let wleft = cx.get_width_left() - 10.;
                    self.tree_text.wrapping = Wrapping::Ellipsis(wleft);
                    self.tree_text.draw_text_walk(cx, name);
                    
                    let (new_scale, new_state) = match state {
                        NodeState::Opening(fac) => {
                            self.view.redraw_view(cx);
                            if *fac < 0.001 {
                                (1.0, NodeState::Open)
                            }
                            else {
                                (1.0 - *fac, NodeState::Opening(*fac * 0.6))
                            }
                        },
                        NodeState::Closing(fac) => {
                            self.view.redraw_view(cx);
                            if *fac < 0.001 {
                                (0.0, NodeState::Closed)
                            }
                            else {
                                (*fac, NodeState::Closing(*fac * 0.6))
                            }
                        },
                        NodeState::Open => {
                            (1.0, NodeState::Open)
                        },
                        NodeState::Closed => {
                            (1.0, NodeState::Closed)
                        }
                    };
                    *state = new_state;
                    last_stack.push(is_last);
                    scale_stack.push(scale * new_scale);
                },
                FileNode::File {name, ..} => {
                    //cx.move_turtle(0., 3.5);
                    cx.turtle_align_y();
                    let wleft = cx.get_width_left() - 10.;
                    self.tree_text.wrapping = Wrapping::Ellipsis(wleft);
                    //cx.realign_turtle(Align::left_center(), false);
                    self.tree_text.color = if is_marked {
                        self.color_tree_folder
                    }
                    else {
                        self.color_tree_file
                    };
                    self.tree_text.draw_text_walk(cx, name);
                }
            }
            
            self.node_bg.end_quad(cx);
            
            cx.turtle_new_line();
            // if any of the parents is closing, don't count alternating lines
            if !file_walker.current_closing() {
                counter += 1;
            }
        }
        
        // draw filler nodes
        if self.row_height > 0. {
            let view_total = cx.get_turtle_bounds();
            let rect_now = cx.get_turtle_rect();
            let mut y = view_total.y;
            while y < rect_now.size.y {
                self.node_bg.color = if counter & 1 == 0 {
                    live_vec4!(cx, self::color_bg_selected)
                }else {
                    live_vec4!(cx, self::color_bg_odd)
                };
                self.node_bg.draw_quad_walk(
                    cx,
                    Walk::wh(Width::Fill, Height::Fix((rect_now.size.y - y).min(self.row_height))),
                );
                cx.turtle_new_line();
                y += self.row_height;
                counter += 1;
            }
        }
        
        // draw the drag item overlay layer if need be
        if let Some(mv) = &self._drag_move {
            if let Ok(()) = self.drag_view.begin_view(cx, Layout {
                abs_origin: Some(Vec2 {x: mv.abs.x + 5., y: mv.abs.y + 5.}),
                ..Default::default()
            }) {
                let mut file_walker = FileWalker::new(&mut self.root_node);
                while let Some((_depth, _index, _len, node)) = file_walker.walk() {
                    let node_draw = if let Some(node_draw) = node.get_draw() {node_draw}else {continue};
                    if node_draw.marked != 0 {
                        //self.drag_bg.z = 10.0;
                        //self.item_draw.tree_text.z = 10.0;
                        self.drag_bg.color = live_vec4!(cx, self::color_drag_bg);
                        self.drag_bg.begin_quad(cx, live_layout!(cx, self::layout_drag_bg));
                        self.tree_text.color = live_vec4!(cx, self::color_tree_folder);
                        self.tree_text.draw_text_walk(cx, match node {
                            FileNode::Folder {name, ..} => {name},
                            FileNode::File {name, ..} => {name}
                        });
                        self.drag_bg.end_quad(cx);
                    }
                }
                self.drag_view.end_view(cx);
            }
        }
        
        self.shadow.draw_shadow_top(cx);
        
        self.view.end_view(cx);
    }
}

#[derive(Clone)]
pub enum NodeState {
    Open,
    Opening(f64),
    Closing(f64),
    Closed
}

#[derive(Clone)]
pub struct NodeDraw {
    area: Area,
    animator: Animator,
    marked: u64
}

#[derive(Clone)]
pub enum FileNode {
    File {name: String, draw: Option<NodeDraw>},
    Folder {name: String, draw: Option<NodeDraw>, state: NodeState, folder: Vec<FileNode>}
}

impl FileNode {
    fn get_draw<'a>(&'a mut self) -> &'a mut Option<NodeDraw> {
        match self {
            FileNode::File {draw, ..} => draw,
            FileNode::Folder {draw, ..} => draw
        }
    }
    
    fn is_open(&self) -> bool {
        match self {
            FileNode::File {..} => false,
            FileNode::Folder {state, ..} => match state {
                NodeState::Opening(..) => true,
                NodeState::Open => true,
                _ => false
            }
        }
    }
    
    fn name(&self) -> String {
        match self {
            FileNode::File {name, ..} => name.clone(),
            FileNode::Folder {name, ..} => name.clone()
        }
    }
}

struct StackEntry<'a> {
    counter: usize,
    index: usize,
    len: usize,
    closing: bool,
    node: &'a mut FileNode
}


pub struct FileWalker<'a>
{
    stack: Vec<StackEntry<'a >>,
}

// this flattens out recursion into an iterator. unfortunately needs unsafe. come on. thats not nice
impl<'a> FileWalker<'a> {
    pub fn new(root_node: &'a mut FileNode) -> FileWalker<'a> {
        return FileWalker {stack: vec![StackEntry {counter: 1, closing: false, index: 0, len: 0, node: root_node}]};
    }
    
    pub fn current_path(&self) -> String {
        // the current stack top returned as path
        let mut path = String::new();
        for i in 0..self.stack.len() {
            if i > 1 {
                path.push_str("/");
            }
            path.push_str(
                &self.stack[i].node.name()
            );
        };
        path
    }
    
    pub fn current_closing(&self) -> bool {
        if let Some(stack_top) = self.stack.last() {
            stack_top.closing
        }
        else {
            false
        }
    }
    
    pub fn walk(&mut self) -> Option<(usize, usize, usize, &mut FileNode)> {
        // lets get the current item on the stack
        let stack_len = self.stack.len();
        let push_or_pop = if let Some(stack_top) = self.stack.last_mut() {
            // return item 'count'
            match stack_top.node {
                FileNode::File {..} => {
                    stack_top.counter += 1;
                    if stack_top.counter == 1 {
                        return Some((stack_len - 1, stack_top.index, stack_top.len, unsafe {std::mem::transmute(&mut *stack_top.node)}));
                    }
                    None // pop stack
                },
                FileNode::Folder {folder, state, ..} => {
                    stack_top.counter += 1;
                    if stack_top.counter == 1 { // return self
                        return Some((stack_len - 1, stack_top.index, stack_top.len, unsafe {std::mem::transmute(&mut *stack_top.node)}));
                    }
                    else {
                        let child_index = stack_top.counter - 2;
                        let opened = if let NodeState::Closed = state {false} else {true};
                        let closing = if let NodeState::Closing(_) = state {true} else {stack_top.closing};
                        if opened && child_index < folder.len() { // child on stack
                            Some(StackEntry {counter: 0, closing: closing, index: child_index, len: folder.len(), node: unsafe {std::mem::transmute(&mut folder[child_index])}})
                        }
                        else {
                            None // pop stack
                        }
                    }
                }
            }
        }
        else {
            None
        };
        if let Some(item) = push_or_pop {
            self.stack.push(item);
            return self.walk();
        }
        else if self.stack.len() > 0 {
            self.stack.pop();
            return self.walk();
        }
        return None;
    }
}
