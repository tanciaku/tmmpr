pub struct StartState {
    pub needs_clear_and_redraw: bool,
}

impl StartState {
    pub fn new() -> StartState {
        StartState {
            needs_clear_and_redraw: true,
        }
    }
    
    pub fn clear_and_redraw(&mut self) {
        self.needs_clear_and_redraw = true;
    }
}