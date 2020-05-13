const W_COEF: f64 = 2.0 / (crate::constants::GAME_WINDOW_WIDTH as f64);
const H_COEF: f64 = 2.0 / (crate::constants::GAME_WINDOW_HEIGHT as f64);

pub fn point_at(x: i32, y: i32) -> [f32; 2] {
    [
        (x as f64 * W_COEF - 1.0) as f32,
        (y as f64 * H_COEF - 1.0) as f32,
    ]
}

pub fn point_unscaled(x: i32, y: i32) -> [f32; 2] {
    [(x as f64 * W_COEF) as f32, (y as f64 * H_COEF) as f32]
}
