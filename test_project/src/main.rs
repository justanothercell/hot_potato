
use hot_potato::{potato, build_and_reload_potatoes};

#[potato(c: u32 = 1)]
fn magicfun(a: u32, b: u32) -> u32 {
    a * b + c
}

#[allow(unused)]
fn main() {
    loop {
        build_and_reload_potatoes().expect("error loading potatoes");
        println!("magicfun(3, 4) = {}", magicfun(3, 4));
        println!("magicfun(5, 8) = {}", magicfun(5, 8));

        let c: u32 = magicfun.get("c");
        println!("{c}");
        magicfun.set("c", c + 1);

        println!("Press enter to hot-reload magicfun");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}