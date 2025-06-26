use rand::Rng;
use core::panic;
use std::cmp::Ordering;
use std::io;

fn main() {
    let mut point = String::new();

    loop{
        io::stdin()
            .read_line(&mut point)
            .expect("Failed to read line");

        let point:Vec<_> = point
            .split(',')
            .map(|s| s.trim())
            .collect();
        let point = match vectofloat(point){
            Some(point) => point,
            None => continue
        };

        
        println!("{:?}",point);
    }



}

fn vectofloat(point: Vec<&str>) -> Option<(f32, f32, f32)> {
    if point.len() != 3 {
        return None;
    }
    let x = point[0].parse::<f32>().ok()?;
    let y = point[1].parse::<f32>().ok()?;
    let z = point[2].parse::<f32>().ok()?;
    Some((x, y, z))
}