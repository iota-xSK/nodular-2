use midi_msg::MidiMsg;
use raylib::{prelude::*, RaylibHandle, RaylibThread};
use rfd::FileDialog;

use crate::graph::{Graph, Node};
use crate::{automaton::Automaton, vec2::Vec2};
use midir::*;
use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::io::Write;

enum Scene {
    Normal,
    MidiSelect,
}

pub struct App {
    pub automaton: Automaton,
    rl: RaylibHandle,
    thread: RaylibThread,
    pub ui_state: UiState,
    clipboard: Option<Graph>,
    connection: Option<MidiOutputConnection>,
    scene: Scene,
    should_step: bool,
}

impl App {
    pub fn new(automaton: Automaton) -> Self {
        let (rl, thread) = raylib::init()
            .size(400, 800)
            .resizable()
            .msaa_4x()
            .title("nodular 2")
            .build();
        Self {
            automaton,
            ui_state: UiState::new(Camera2D {
                offset: Vector2::zero(),
                target: Vector2 { x: 0.0, y: 0.0 },
                rotation: 0.0,
                zoom: 1.0,
            }),
            rl,
            thread,
            clipboard: None,
            scene: Scene::Normal,
            connection: None,
            should_step: false,
        }
    }

    fn step(&mut self) {
        self.play_midi();
        self.automaton.step();
    }

    pub fn run(&mut self) {
        self.rl.set_target_fps(60);
        while !self.rl.window_should_close() {
            match self.scene {
                Scene::Normal => {
                    if (self.rl.get_time() % 0.5) < self.rl.get_frame_time() as f64
                        && self.ui_state.playing
                    {
                        self.step();
                    } else if self.should_step {
                        self.step()
                    }

                    // pause unpuase
                    if self.rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        self.ui_state.playing = !self.ui_state.playing;
                    }
                    // find coliding node
                    self.ui_state.hovering_over = None;
                    for (i, position) in self
                        .automaton
                        .graph
                        .nodes
                        .iter()
                        .map(|a| a.position)
                        .enumerate()
                    {
                        if check_collision_point_circle(
                            self.rl.get_screen_to_world2D(
                                self.rl.get_mouse_position(),
                                self.ui_state.camera,
                            ),
                            <Vec2 as Into<Vector2>>::into(position),
                            30.0 * self.ui_state.camera.zoom,
                        ) {
                            self.ui_state.hovering_over = Some(i)
                        }
                    }

                    // add node
                    println!("{:?}", self.ui_state.selected_state);
                    if self.rl.is_key_pressed(KeyboardKey::KEY_A) {
                        self.automaton.graph.add_node(Node::new(
                            self.ui_state.selected_state as u32,
                            self.ui_state.selected_state as u32,
                            vec![],
                            self.rl
                                .get_screen_to_world2D(
                                    self.rl.get_mouse_position(),
                                    self.ui_state.camera,
                                )
                                .into(),
                        ));
                    }

                    // connect nodes
                    if self
                        .rl
                        .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
                    {
                        self.ui_state.connecting_from = self.ui_state.hovering_over;
                    }
                    if self
                        .rl
                        .is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT)
                    {
                        if let (Some(hovering), Some(from)) =
                            (self.ui_state.hovering_over, self.ui_state.connecting_from)
                        {
                            if from != hovering {
                                if self.automaton.graph.nodes[hovering].edges.contains(&from) {
                                    self.automaton.graph.remove_edge(hovering, from);
                                } else {
                                    self.automaton.graph.add_edge(hovering, from);
                                }
                            }
                        }
                        self.ui_state.connecting_from = None;
                    }
                    self.control_camera();
                    self.render();

                    // selecting nodes

                    if self.rl.get_mouse_x() > 100 {
                        if self
                            .rl
                            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                        {
                            if let Some(hovering) = self.ui_state.hovering_over {
                                if self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                                    if !self.ui_state.selected.contains(&hovering) {
                                        self.ui_state.selected.push(hovering)
                                    }
                                } else {
                                    self.ui_state.selected = vec![hovering]
                                }
                            } else {
                            }
                        }
                    }

                    // moving nodes
                    if self
                        .rl
                        .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
                    {
                        self.ui_state.dragging_node_positions = None;
                    }

                    if self
                        .rl
                        .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                    {
                        self.ui_state.click_position = self.rl.get_mouse_position().into();
                        self.ui_state.dragging_node_positions = Some(
                            self.automaton
                                .graph
                                .nodes
                                .iter()
                                .map(|a| a.position)
                                .collect(),
                        );
                    }
                    if self.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                        for selected in &self.ui_state.selected {
                            if let Some(dragging) = self.ui_state.dragging_node_positions.clone() {
                                self.automaton.graph.nodes[*selected].position = dragging[*selected]
                                    + self
                                        .rl
                                        .get_screen_to_world2D(
                                            self.rl.get_mouse_position(),
                                            self.ui_state.camera,
                                        )
                                        .into()
                                    - self
                                        .rl
                                        .get_screen_to_world2D(
                                            <Vec2 as Into<Vector2>>::into(
                                                self.ui_state.click_position,
                                            ),
                                            self.ui_state.camera,
                                        )
                                        .into()
                            } else {
                                println!("error")
                            }
                        }
                    }
                    // deleting nodes
                    if self.rl.is_key_pressed(KeyboardKey::KEY_DELETE) {
                        while let Some(node) = self.ui_state.selected.pop() {
                            self.automaton
                                .graph
                                .remove_node_from_app(node, &mut self.ui_state);
                        }
                        self.ui_state.selected = vec![];
                    }
                    // changing state
                    if self.rl.is_key_pressed(KeyboardKey::KEY_S) {
                        for node in &self.ui_state.selected {
                            self.automaton.graph.nodes[*node].write =
                                self.ui_state.selected_state as u32
                        }
                    }
                    // box select
                    if self.rl.get_mouse_x() > 100 {
                        if self
                            .rl
                            .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
                        {
                            if let Some(box_drag_start) = self.ui_state.box_select_corner {
                                let rect = find_rect(
                                    box_drag_start.into(),
                                    self.rl.get_screen_to_world2D(
                                        self.rl.get_mouse_position(),
                                        self.ui_state.camera,
                                    ),
                                );

                                let x_1 = rect.x;
                                let y_1 = rect.y;
                                let x_2 = rect.width + rect.x;
                                let y_2 = rect.height + rect.y;

                                if !self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                                    self.ui_state.selected = vec![];
                                }
                                for (i, position) in self
                                    .automaton
                                    .graph
                                    .nodes
                                    .iter()
                                    .map(|a| a.position)
                                    .enumerate()
                                {
                                    if position.x >= x_1
                                        && position.x <= x_2
                                        && position.y >= y_1
                                        && position.y <= y_2
                                    {
                                        self.ui_state.selected.push(i)
                                    }
                                }
                            }
                        }
                    }

                    if let None = self.ui_state.hovering_over {
                        if self
                            .rl
                            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                        {
                            self.ui_state.box_select_corner = Some(
                                self.rl
                                    .get_screen_to_world2D(
                                        self.rl.get_mouse_position(),
                                        self.ui_state.camera,
                                    )
                                    .into(),
                            );
                        }
                    }
                    if self
                        .rl
                        .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
                    {
                        self.ui_state.box_select_corner = None
                    }

                    if self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                        if let Some(number) = self.rl.get_key_pressed_number() {
                            if (number as i32) - 48 <= self.automaton.rules.names.len() as i32
                                && (number as i32 - 48) >= 0
                            {
                                self.ui_state.selected_state = (number as i32) - 49
                            }
                        }
                    }
                    // copy
                    //
                    if self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                        && self.rl.is_key_pressed(KeyboardKey::KEY_C)
                    {
                        let graph = Graph::copy(&self.automaton.graph, &self.ui_state.selected);
                        self.clipboard = Some(graph);

                        println!("{:?}", self.clipboard);
                    }
                    // // paste
                    if self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                        && self.rl.is_key_pressed(KeyboardKey::KEY_V)
                    {
                        self.ui_state.selected = vec![];
                        let len = self.automaton.graph.nodes.len();
                        if let Some(clipboard) = self.clipboard.clone() {
                            for node in clipboard.nodes {
                                let new_node = Node::new(
                                    node.read,
                                    node.write,
                                    node.edges.iter().map(|a| a + len).collect(),
                                    node.position + Vec2::new(50.0, 50.0),
                                );

                                self.automaton.graph.add_node(new_node);
                                self.ui_state
                                    .selected
                                    .push(self.automaton.graph.nodes.len() - 1)
                            }
                        }
                    }
                }
                Scene::MidiSelect => {
                    self.midi_select();
                }
            }
        }
    }

    fn midi_select(&mut self) {
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::RAYWHITE);
        if d.gui_button(rrect(0, 0, 100, 30), "back to app") {
            self.scene = Scene::Normal;
        }
        match midir::MidiOutput::new("nodular-2") {
            Ok(some) => {
                let ports = &some.ports();

                for (i, port) in ports.iter().enumerate() {
                    if let Ok(name) = some.port_name(port) {
                        if d.gui_button(
                            rrect(
                                d.get_screen_width() / 2 - 300,
                                (200 + i * 30) as f32,
                                300,
                                30,
                            ),
                            &name,
                        ) {
                            let possible_connection = some.connect(&ports[i], "nodular-2");
                            if let Ok(connection) = possible_connection {
                                self.connection = Some(connection)
                            }
                            break;
                        }
                    }
                }
            }
            Err(_) => d.draw_text(
                "Midi initialisation error",
                d.get_screen_width() / 2,
                d.get_screen_height() / 2,
                30,
                Color::BLACK,
            ),
        }
    }

    fn control_camera(&mut self) {
        self.ui_state.camera.offset = Vector2::new(
            self.rl.get_screen_width() as f32 / 2.0,
            self.rl.get_screen_height() as f32 / 2.0,
        );
        if self.rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            || self.rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
        {
            self.ui_state.camera.zoom += self.rl.get_mouse_wheel_move() * 0.05;
        } else if !self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            self.ui_state.camera.target.y +=
                self.rl.get_mouse_wheel_move() * 50.0 * self.ui_state.camera.zoom;
        } else {
            self.ui_state.camera.target.x -=
                self.rl.get_mouse_wheel_move() * 50.0 * self.ui_state.camera.zoom;
        }
    }
    fn render(&mut self) {
        let node_positions: Vec<Vector2> = self
            .automaton
            .graph
            .nodes
            .iter()
            .map(|a| a.position)
            .map(|a| {
                self.rl
                    .get_world_to_screen2D(<Vec2 as Into<Vector2>>::into(a), self.ui_state.camera)
            })
            .collect();

        let height = self.rl.get_screen_height();
        let _width = self.rl.get_screen_width();
        let _mouse_position = self.rl.get_mouse_position();

        let mut select_rect = None;

        if let Some(drag_box) = self.ui_state.box_select_corner {
            select_rect = Some(find_rect(
                self.rl
                    .get_world_to_screen2D(
                        <Vec2 as Into<Vector2>>::into(drag_box),
                        self.ui_state.camera,
                    )
                    .into(),
                _mouse_position,
            ));
        }

        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::color_from_hsv(0.5, 0.1, 1.0));

        // render canvas
        //
        // selection_circles
        for selected in &self.ui_state.selected {
            d.draw_circle_v(
                node_positions[*selected],
                32.0 * self.ui_state.camera.zoom,
                Color::BLACK,
            )
        }

        // nodes
        for (node, position) in self
            .automaton
            .graph
            .nodes
            .iter()
            .map(|a| a.write)
            .zip(&node_positions)
        {
            d.draw_circle_v(
                position,
                30.0 * self.ui_state.camera.zoom,
                Color::color_from_hsv(distribute_hue(node), 0.5, 0.90),
            )
        }
        // connections
        for i in 0..self.automaton.graph.nodes.len() {
            for edge in &self.automaton.graph.nodes[i].edges {
                draw_spring_arrow(
                    &mut d,
                    node_positions[*edge].into(),
                    node_positions[i].into(),
                    Color::BLACK,
                    30.0 * self.ui_state.camera.zoom,
                )
            }
        }

        // box select
        if let Some(rect) = select_rect {
            d.draw_rectangle_rec(rect, Color::new(10, 10, 255, 40))
        }

        // render gui

        d.gui_panel(rrect(0, 0, 100, height), "");
        d.gui_panel(rrect(0, 0, _width, 30), "");

        d.gui_check_box(
            rrect(10, 10, 10, 10),
            "playing",
            &mut self.ui_state.playing,
        );

        let mut strings = vec![];
        for name in &self.automaton.rules.names {
            strings.push(name.clone());
        }
        if d.gui_button(rrect(0, 30, 100, 30), "step") {
            self.should_step = true;
        } else {
            self.should_step = false;
        }
        d.gui_list_view_ex(
            rrect(0, 60, 100, 300.min(height - 60)),
            strings.iter(),
            &mut 1,
            &mut self.ui_state.selected_state,
            &mut self.ui_state.type_scroll,
        );

        if d.gui_button(rrect(100, 0, 100, 30), "open world") {
            if let Some(file) = FileDialog::new().pick_file() {
                if let Ok(content) = fs::read_to_string(file) {
                    if let Ok(deserialized) = serde_json::from_str(&content) {
                        self.automaton = deserialized;
                    } else {
                        println!("unable to deserialize file")
                    }
                } else {
                    println!("unable to read file")
                }
            } else {
                println!("unable to pick file")
            }
        }
        if d.gui_button(rrect(200, 0, 100, 30), "save_world") {
            if let Some(file_choice) = FileDialog::new().save_file() {
                if let Ok(mut file) = File::create(file_choice) {
                    if let Ok(parsed) = serde_json::to_string_pretty(&self.automaton) {
                        file.write_all(parsed.as_bytes())
                            .unwrap_or_else(|_| println!("unable to write to file"));
                    } else {
                        println!("unable to parse file")
                    }
                } else {
                    println!("unable to create file")
                }
            } else {
                println!("unable to pick file to create")
            }
        }

        if d.gui_button(rrect(300, 0, 100, 30), "Midi settings") {
            self.scene = Scene::MidiSelect;
        }

        note_input_box(
            &mut d,
            rrect(0, 360, 40, 30),
            &mut self.ui_state.note,
            &mut self.ui_state.node_edit_mode,
        );

        if d.gui_button(rrect(40, 360, 60, 30), "note") {
            for selected in &self.ui_state.selected {
                self.automaton.graph.nodes[*selected].note = Some(self.ui_state.note.clone())
            }
        }
        if d.gui_button(rrect(0, 390, 100, 30), "clear note") {
            for selected in &self.ui_state.selected {
                self.automaton.graph.nodes[*selected].note = None;
            }
        }
    }
    pub fn play_midi(&mut self) {
        if let Some(output) = &mut self.connection {
            for node in &self.automaton.graph.nodes {
                if let Some(note) = &node.note {
                    if node.read == 1 && node.write == 0 {
                        if let Err(err) = output.send(&note.to_midi_on()) {
                            println!("{:?}", err)
                        }
                    } else if node.read == 0 && node.write == 1 {
                        if let Err(err) = output.send(&note.to_midi_off()) {
                            println!("{:?}", err)
                        }
                    }
                }
            }
        }
    }
}

pub struct UiState {
    pub camera: Camera2D,
    pub playing: bool,
    pub selected_state: i32,
    pub type_scroll: i32,
    pub connecting_from: Option<usize>,
    pub hovering_over: Option<usize>,
    pub selected: Vec<usize>,
    pub click_position: Vec2,
    pub dragging_node_positions: Option<Vec<Vec2>>,
    pub box_select_corner: Option<Vec2>,
    pub selected_midi_value: i32,
    pub node_edit_mode: bool,
    pub note: Note,
}

impl UiState {
    fn new(camera: Camera2D) -> Self {
        Self {
            camera,
            playing: false,
            selected_state: 0,
            type_scroll: 0,
            connecting_from: None,
            hovering_over: None,
            selected: vec![],
            click_position: Vec2::new(0.0, 0.0),
            dragging_node_positions: None,
            box_select_corner: None,
            selected_midi_value: 0,
            node_edit_mode: false,
            note: Note {
                letter: NoteLetter::C,
                accidental: Accidental::Neutral,
                octave: 4,
            },
        }
    }
}

fn distribute_hue(index: u32) -> f32 {
    let golden_ratio_conjugate = 0.618033988749895;

    let hue = ((index as f32 * golden_ratio_conjugate) % 1.0) * 360.0;

    hue
}

fn draw_spring_arrow(d: &mut RaylibDrawHandle, start: Vec2, end: Vec2, color: Color, radius: f32) {
    // Calculate arrowhead size based on spring length
    let spring_length = (end - start).length();
    let arrowhead_size = 5.0;

    // Calculate normalized direction vector of the spring
    let direction = (end - start).normalized();

    // Calculate perpendicular vector to the spring direction
    let perpendicular = Vec2 {
        x: -direction.y,
        y: direction.x,
    };

    let control_point = (start + end) / 2.0 + perpendicular * (end - start).length() * 0.2;

    let arrow_perpendicular = Vec2 {
        x: -direction.y,
        y: direction.x,
    };

    // Calculate arrowhead points
    let arrowhead_left =
        end - (direction * arrowhead_size) + (arrow_perpendicular * arrowhead_size);
    let arrowhead_right =
        end - (direction * arrowhead_size) - (arrow_perpendicular * arrowhead_size);

    // Draw arrowhead triangle
    d.draw_triangle(
        <Vec2 as Into<Vector2>>::into(arrowhead_left - direction * radius),
        <Vec2 as Into<Vector2>>::into(end - direction * radius),
        <Vec2 as Into<Vector2>>::into(arrowhead_right - direction * radius),
        color,
    );

    d.draw_line_v(
        <Vec2 as Into<Vector2>>::into(start + direction * radius),
        <Vec2 as Into<Vector2>>::into(end - direction * radius),
        color,
    );
}

fn find_rect(corner_1: Vector2, corner_2: Vector2) -> Rectangle {
    Rectangle::new(
        corner_1.x.min(corner_2.x),
        corner_1.y.min(corner_2.y),
        (corner_2.x - corner_1.x).abs(),
        (corner_2.y - corner_1.y).abs(),
    )
}

#[derive(Clone, Debug, Copy, serde::Serialize, serde::Deserialize)]
enum NoteLetter {
    C = 0,
    D = 2,
    E = 4,
    F = 5,
    G = 7,
    A = 9,
    B = 11,
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum Accidental {
    Flat = -1,
    Neutral = 0,
    Sharp = 1,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Note {
    letter: NoteLetter,
    accidental: Accidental,
    octave: u8,
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note = match self.letter {
            NoteLetter::C => 'C',
            NoteLetter::D => 'D',
            NoteLetter::E => 'E',
            NoteLetter::F => 'F',
            NoteLetter::G => 'G',
            NoteLetter::A => 'A',
            NoteLetter::B => 'B',
        };
        write!(f, "{}", note)?;
        match self.accidental {
            Accidental::Flat => write!(f, "b")?,
            Accidental::Neutral => (),
            Accidental::Sharp => write!(f, "#")?,
        };
        write!(f, "{}", self.octave)?;
        Ok(())
    }
}
impl Debug for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note = match self.letter {
            NoteLetter::C => 'C',
            NoteLetter::D => 'D',
            NoteLetter::E => 'E',
            NoteLetter::F => 'F',
            NoteLetter::G => 'G',
            NoteLetter::A => 'A',
            NoteLetter::B => 'B',
        };
        write!(f, "{}", note)?;
        match self.accidental {
            Accidental::Flat => write!(f, "b")?,
            Accidental::Neutral => (),
            Accidental::Sharp => write!(f, "#")?,
        };
        write!(f, "{}", self.octave)?;
        Ok(())
    }
}

impl Note {
    fn to_midi_number(&self) -> u8 {
        24 + self.letter as u8 + self.octave * 12
    }
    fn to_midi_on(&self) -> Vec<u8> {
        MidiMsg::ChannelVoice {
            channel: midi_msg::Channel::Ch1,
            msg: midi_msg::ChannelVoiceMsg::NoteOn {
                note: self.to_midi_number(),
                velocity: 60,
            },
        }
        .to_midi()
    }
    fn to_midi_off(&self) -> Vec<u8> {
        MidiMsg::ChannelVoice {
            channel: midi_msg::Channel::Ch1,
            msg: midi_msg::ChannelVoiceMsg::NoteOff {
                note: self.to_midi_number(),
                velocity: 0,
            },
        }
        .to_midi()
    }
}

fn note_input_box(
    d: &mut RaylibDrawHandle,
    rect: impl Into<Rectangle>,
    note: &mut Note,
    edit_mode: &mut bool,
) {
    let rect = rect.into();
    let mouse = d.get_mouse_position();
    let hovering = mouse.x > rect.x
        && mouse.y > rect.y
        && mouse.x < rect.x + rect.width
        && mouse.y < rect.y + rect.height;
    if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        if hovering {
            *edit_mode = true
        } else {
            *edit_mode = false
        }
    }
    d.draw_rectangle_rec(
        rect,
        if *edit_mode {
            Color::from_hex("97e8ff").unwrap()
        } else if !hovering {
            Color::LIGHTGRAY
        } else {
            Color::from_hex("c9effe").unwrap()
        },
    );
    let line_color = if *edit_mode {
        Color::from_hex("0492c7").unwrap()
    } else if !hovering {
        Color::GRAY
    } else {
        Color::SKYBLUE
    };
    d.draw_rectangle_lines_ex(rect, 2.0, line_color);

    if *edit_mode {
        if d.is_key_pressed(KeyboardKey::KEY_C) {
            note.letter = NoteLetter::C
        } else if d.is_key_pressed(KeyboardKey::KEY_D) {
            note.letter = NoteLetter::D
        } else if d.is_key_pressed(KeyboardKey::KEY_E) {
            note.letter = NoteLetter::E
        } else if d.is_key_pressed(KeyboardKey::KEY_F) {
            note.letter = NoteLetter::F
        } else if d.is_key_pressed(KeyboardKey::KEY_G) {
            note.letter = NoteLetter::G
        } else if d.is_key_pressed(KeyboardKey::KEY_A) {
            note.letter = NoteLetter::A
        } else if d.is_key_pressed(KeyboardKey::KEY_B) {
            note.letter = NoteLetter::B
        }

        if d.is_key_pressed(KeyboardKey::KEY_UP) {
            note.accidental = Accidental::Sharp
        } else if d.is_key_pressed(KeyboardKey::KEY_DOWN) {
            note.accidental = Accidental::Flat
        } else if d.is_key_pressed(KeyboardKey::KEY_PERIOD) {
            note.accidental = Accidental::Neutral
        }

        if d.is_key_pressed(KeyboardKey::KEY_ZERO) {
            note.octave = 0;
        } else if d.is_key_pressed(KeyboardKey::KEY_ONE) {
            note.octave = 1;
        } else if d.is_key_pressed(KeyboardKey::KEY_TWO) {
            note.octave = 2;
        } else if d.is_key_pressed(KeyboardKey::KEY_THREE) {
            note.octave = 3;
        } else if d.is_key_pressed(KeyboardKey::KEY_FOUR) {
            note.octave = 4;
        } else if d.is_key_pressed(KeyboardKey::KEY_FIVE) {
            note.octave = 5;
        } else if d.is_key_pressed(KeyboardKey::KEY_SIX) {
            note.octave = 6;
        } else if d.is_key_pressed(KeyboardKey::KEY_SEVEN) {
            note.octave = 7;
        } else if d.is_key_pressed(KeyboardKey::KEY_EIGHT) {
            note.octave = 8;
        } else if d.is_key_pressed(KeyboardKey::KEY_NINE) {
            note.octave = 9;
        }
    }

    let text = format!("{}", *note);
    d.draw_text(
        &text,
        (rect.x + rect.width / 2.0) as i32 - text.len() as i32 * 4,
        (rect.y + rect.height / 2.0) as i32 - 7,
        15,
        line_color,
    )
}
