use hot_potato::{potato, build_and_reload_potatoes};

#[potato]
fn magicfun(a: u32, b: u32) -> u32 {
    a + b
}

fn main() {
    build_and_reload_potatoes();
    println!("Hello, world!");
    loop {
        
    }
}