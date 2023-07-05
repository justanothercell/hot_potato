use hot_potato::potato;

#[potato]
fn magicfun(a: u32, b: u32) -> u32 {
    a + b
}

fn main() {
    println!("{}", magicfun(1, 1));
    println!("{}", magicfun(3, 3));
    println!("{}", magicfun(4, 7));
}