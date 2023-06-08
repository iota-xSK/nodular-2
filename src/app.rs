use raylib::{prelude::*, RaylibHandle, RaylibThread};

use crate::{automaton::Automaton, vec2::Vec2};
use std::borrow::Borrow;
use std::ffi::{CStr, CString};

pub struct App {
    automaton: VisualAutomaton,
    rl: RaylibHandle,
    thread: RaylibThread,
    ui_state: UiState,
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
        }
    }

    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
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
                    self.automaton.automaton.graph.add_edge(from, hovering);
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
                    node_positions[i].into(),
                    node_positions[*edge].into(),
                    Color::BLACK,
                    30.0 * self.ui_state.camera.zoom,
                )
            }
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
    }
}

struct UiState {
    camera: Camera2D,

    playing: bool,

    selected_state: i32,
    type_scroll: i32,

    connecting_from: Option<usize>,
    hovering_over: Option<usize>,
    selected: Vec<usize>,

    click_position: Vec2,
    dragging_node_positions: Option<Vec<Vec2>>,
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
        }
    }
}

pub struct VisualAutomaton {
    automaton: Automaton,
    node_possions: Vec<Vec2>,
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
