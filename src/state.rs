// UI と ロジックの両方から参照される状態

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SelectionRect {
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
    pub is_dragging: bool,
}

impl SelectionRect {
    pub fn left(&self) -> f32 {
        self.start_x.min(self.current_x)
    }

    pub fn top(&self) -> f32 {
        self.start_y.min(self.current_y)
    }

    pub fn right(&self) -> f32 {
        self.start_x.max(self.current_x)
    }

    pub fn bottom(&self) -> f32 {
        self.start_y.max(self.current_y)
    }

    pub fn width(&self) -> f32 {
        (self.current_x - self.start_x).abs()
    }

    pub fn height(&self) -> f32 {
        (self.current_y - self.start_y).abs()
    }

    pub fn to_egui_rect(&self) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(self.left(), self.top()),
            egui::pos2(self.right(), self.bottom()),
        )
    }
}
