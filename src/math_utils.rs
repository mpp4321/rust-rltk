use rand::Rng;

pub fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn chance(f: f32) -> bool {
    return rand::thread_rng().gen::<f32>() < f;
}

#[allow(dead_code)]
pub fn random_point(x1: i32, x2: i32, y1: i32, y2: i32) -> (i32, i32) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(x1..x2);
    let y = rng.gen_range(y1..y2);
    return (x, y);
}
