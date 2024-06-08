use std::fmt::Debug;
use std::ops::Add;
#[derive(Debug)]
struct Point {
    x: u32,
    y: u32,
}
impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        return Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}
#[tokio::main]
async fn main() {
    let point_a = Point { x: 1, y: 2 };
    let point_b = Point { x: 3, y: 6 };
    let new_point = point_a + point_b;
    println!("{:?}", new_point);
}
