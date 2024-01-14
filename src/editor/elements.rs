use eframe::{epaint::{Pos2, Color32, Stroke}, egui::{Painter, Rect}};

pub trait Position {
    fn set_position(&mut self, position: Pos2);
    fn get_position(&self) -> Pos2;
}

pub struct EditorInput {
    position: Pos2,
}

pub struct EditorOutput {
    position: Pos2,
}

pub struct EditorComponent {
    position: Pos2,
}

pub struct EditorLine {
    start: Pos2,
    end: Pos2,
}

impl EditorInput {
    pub fn new(position: Pos2) -> Self {
        Self { position }
    }

    pub fn draw(&self, painter: &Painter, area: Rect) {
        painter.circle(self.position, 20.0, Color32::GREEN, Stroke::new(5.0, Color32::DARK_GREEN));
    }
}

impl EditorOutput {
    pub fn new(position: Pos2) -> Self {
        Self { position }
    }
}

impl EditorComponent {
    pub fn new(position: Pos2) -> Self {
        Self { position }
    }
}

impl EditorLine {
    pub fn new(start: Pos2, end: Pos2) -> Self {
        Self { start, end }
    }
}