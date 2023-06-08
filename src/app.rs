use raylib::{prelude::*, RaylibHandle, RaylibThread};
use rfd::FileDialog;

use crate::clipboard::Clipboard;
use crate::graph::Graph;
use crate::{automaton::Automaton, vec2::Vec2};
use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::Write;

pub struct App {
    pub automaton: VisualAutomaton,
    rl: RaylibHandle,
    thread: RaylibThread,
    pub ui_state: UiState,
    clipboard: Option<Clipboard>,
}

impl App {
    pub fn new(automaton: VisualAutomaton, camera: Camera2D) -> Self {
        let (rl, thread) = raylib::init()
            .size(400, 800)
            .resizable()
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
        }
    }

    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
            if (self.rl.get_time() % 0.5) < self.rl.get_frame_time() as f64 && self.ui_state.playing
            {
                self.automaton.automaton.step();
            }

            // pause unpuase
            if self.rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                self.ui_state.playing = !self.ui_state.playing;
            }
            // find coliding node
            self.ui_state.hovering_over = None;
            for (i, position) in self.automaton.node_possions.iter().enumerate() {
                if check_collision_point_circle(
                    self.rl
                        .get_screen_to_world2D(self.rl.get_mouse_position(), self.ui_state.camera),
                    <Vec2 as Into<Vector2>>::into(*position),
                    30.0 * self.ui_state.camera.zoom,
                ) {
                    self.ui_state.hovering_over = Some(i)
                }
            }

            // add node
            if self.rl.is_key_pressed(KeyboardKey::KEY_A) {
                self.automaton.add_node(
                    self.ui_state.selected_state as u32,
                    self.rl
                        .get_screen_to_world2D(self.rl.get_mouse_position(), self.ui_state.camera)
                        .into(),
                );
            }

            // connect nodes
            if self
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON)
            {
                self.ui_state.connecting_from = self.ui_state.hovering_over;
            }
            if self
                .rl
                .is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON)
            {
                if let (Some(hovering), Some(from)) =
                    (self.ui_state.hovering_over, self.ui_state.connecting_from)
                {
                    if from != hovering {
                        if self.automaton.automaton.graph.edges[hovering].contains(&from) {
                            self.automaton.automaton.graph.remove_edge(hovering, from);
                        } else {
                            self.automaton.automaton.graph.add_edge(hovering, from);
                        }
                    }
                }
                self.ui_state.connecting_from = None;
            }
            self.control_camera();
            self.render();

            // selecting nodes

            if self
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON)
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
                    self.ui_state.selected = vec![];
                }
            }

            // moving nodes
            if self
                .rl
                .is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
            {
                self.ui_state.dragging_node_positions = None;
            }

            if self
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON)
            {
                self.ui_state.click_position = self.rl.get_mouse_position().into();
                self.ui_state.dragging_node_positions = Some(self.automaton.node_possions.clone());
            }
            if self.rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
                for selected in &self.ui_state.selected {
                    if let Some(dragging) = self.ui_state.dragging_node_positions.clone() {
                        self.automaton.node_possions[*selected] = dragging[*selected]
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
                                    <Vec2 as Into<Vector2>>::into(self.ui_state.click_position),
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
                for node in &self.ui_state.selected {
                    self.automaton.automaton.graph.remove_node(*node);
                }
                self.ui_state.selected = vec![];
            }
            // changing state
            if self.rl.is_key_pressed(KeyboardKey::KEY_S) {
                for node in &self.ui_state.selected {
                    self.automaton.automaton.graph.nodes_write[*node] =
                        Some(self.ui_state.selected_state as u32)
                }
            }
            // box select
            if self
                .rl
                .is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
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
                    for (i, position) in self.automaton.node_possions.iter().enumerate() {
                        if position.x >= x_1
                            && position.x <= x_2
                            && position.y >= y_1
                            && position.y <= y_2
                        {
                            if let Some(_) = self.automaton.automaton.graph.nodes_read[i] {
                                self.ui_state.selected.push(i)
                            }
                        }
                    }
                }
            }

            if let None = self.ui_state.hovering_over {
                if self
                    .rl
                    .is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON)
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
                .is_mouse_button_released(MouseButton::MOUSE_LEFT_BUTTON)
            {
                self.ui_state.box_select_corner = None
            }
            // copy

            if self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                && self.rl.is_key_pressed(KeyboardKey::KEY_C)
            {
                // let graph = self.automaton.automaton.graph.copy(&self.ui_state.selected);
                let graph = Clipboard::copy(
                    &self.automaton.automaton.graph,
                    &self.ui_state.selected,
                    &self.automaton.node_possions,
                );
                println!("{:?}", graph);

                self.clipboard = Some(graph)
            }
            // paste
            if self.rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                && self.rl.is_key_pressed(KeyboardKey::KEY_V)
            {
                if let Some(clipboard) = self.clipboard.clone() {
                    clipboard.paste(self);
                }
            }
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
            .node_possions
            .iter()
            .map(|a| {
                self.rl
                    .get_world_to_screen2D(<Vec2 as Into<Vector2>>::into(*a), self.ui_state.camera)
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
            .automaton
            .graph
            .nodes_write
            .iter()
            .zip(&node_positions)
        {
            if let Some(node) = node {
                d.draw_circle_v(
                    position,
                    30.0 * self.ui_state.camera.zoom,
                    Color::color_from_hsv(distribute_hue(*node), 0.5, 0.90),
                )
            }
        }
        // connections
        for i in 0..self.automaton.automaton.graph.edges.len() {
            for edge in &self.automaton.automaton.graph.edges[i] {
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

        d.gui_panel(rrect(0, 0, 100, height));

        self.ui_state.playing = d.gui_check_box(
            rrect(10, 10, 10, 10),
            Some(rstr!("playing")),
            self.ui_state.playing,
        );

        let mut strings = vec![];
        for name in &self.automaton.automaton.rules.names {
            strings.push(CString::new(name.clone()).unwrap());
        }
        self.ui_state.selected_state = d.gui_list_view_ex(
            rrect(0, 25, 100, 300.min(height - 60)),
            &strings.iter().map(|a| a.borrow()).collect::<Vec<&CStr>>(),
            &mut 1,
            &mut self.ui_state.type_scroll,
            // &mut self.selected_state,
            self.ui_state.selected_state,
        );

        if d.gui_button(rrect(0, 365, 100, 30), Some(rstr!("open world"))) {
            if let Some(file) = FileDialog::new().pick_file() {
                if let Ok(content) = fs::read_to_string(file) {
                    if let Ok(deserialized) = serde_json::from_str(&content) {
                        self.automaton = deserialized;
                    }
                } else {
                    println!("unable to read file")
                }
            } else {
                println!("unable to pick file")
            }
        }
        if d.gui_button(rrect(0, 400, 100, 30), Some(rstr!("save_world"))) {
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

        if d.gui_button(rrect(0, 435, 100, 30), Some(rstr!("step"))) {
            self.automaton.automaton.step();
        }
    }
}

pub struct UiState {
    camera: Camera2D,

    playing: bool,

    selected_state: i32,
    type_scroll: i32,

    connecting_from: Option<usize>,
    hovering_over: Option<usize>,
    selected: Vec<usize>,

    click_position: Vec2,
    dragging_node_positions: Option<Vec<Vec2>>,

    box_select_corner: Option<Vec2>,
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
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualAutomaton {
    pub automaton: Automaton,
    pub node_possions: Vec<Vec2>,
}

impl VisualAutomaton {
    pub fn new(automaton: Automaton, node_possions: Vec<Vec2>) -> Self {
        Self {
            automaton,
            node_possions,
        }
    }
    fn add_node(&mut self, state: u32, position: Vec2) -> usize {
        let i = self.automaton.graph.add_node(state);
        if self.node_possions.len() > i {
            self.node_possions[i] = position;
        } else {
            self.node_possions.push(position)
        }
        i
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
    let arrowhead_size = spring_length / 20.0;

    // Calculate normalized direction vector of the spring
    let direction = (end - start).normalized();

    // Calculate perpendicular vector to the spring direction
    let perpendicular = Vec2 {
        x: -direction.y,
        y: direction.x,
    };

    let control_point = (start + end) / 2.0 + perpendicular * (end - start).length() * 0.2;

    let arrow_direction = (end - control_point).normalized();
    let arrow_perpendicular = Vec2 {
        x: -arrow_direction.y,
        y: arrow_direction.x,
    };

    // Calculate arrowhead points
    let arrowhead_left =
        end - (arrow_direction * arrowhead_size) + (arrow_perpendicular * arrowhead_size);
    let arrowhead_right =
        end - (arrow_direction * arrowhead_size) - (arrow_perpendicular * arrowhead_size);

    // Draw arrowhead triangle
    d.draw_triangle(
        <Vec2 as Into<Vector2>>::into(arrowhead_left - direction * radius),
        <Vec2 as Into<Vector2>>::into(end - direction * radius),
        <Vec2 as Into<Vector2>>::into(arrowhead_right - direction * radius),
        color,
    );

    d.draw_line_bezier_quad(
        <Vec2 as Into<Vector2>>::into(start + direction * radius),
        <Vec2 as Into<Vector2>>::into(end - direction * radius),
        <Vec2 as Into<Vector2>>::into(control_point),
        1.0,
        color,
    )
}

fn find_rect(corner_1: Vector2, corner_2: Vector2) -> Rectangle {
    Rectangle::new(
        corner_1.x.min(corner_2.x),
        corner_1.y.min(corner_2.y),
        (corner_2.x - corner_1.x).abs(),
        (corner_2.y - corner_1.y).abs(),
    )
}
