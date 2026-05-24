// UI と ロジックの両方から参照される状態

// 選択領域
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SelectionRect {
    pub start_x: f64,
    pub start_y: f64,
    pub current_x: f64,
    pub current_y: f64,
    pub is_dragging:bool,
}

impl SelectionRect {
    pub fn left(&self) -> f64 {
        self.start_x.min(self.current_x)
    }

    pub fn top(&self) -> f64 {
        self.start_y.min(self.current_y)
    }

    pub fn width(&self) -> f64 {
        (self.current_x - self.start_x).abs()
    }

    pub fn height(&self) -> f64 {
        (self.current_y - self.start_y).abs()
    }
}
