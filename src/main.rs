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
    docker_warp::docker::docker().await;
}
