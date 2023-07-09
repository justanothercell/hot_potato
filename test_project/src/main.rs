use std::io::{stdout, stdin, Write};

use hot_potato::{potato, build_and_reload_potatoes};

#[potato(c: u32 = 1)]
fn magicfun(a: u32, b: u32) -> u32 {
    a * b * c
}

#[allow(unused)]
fn main() {
    build_and_reload_potatoes().expect("error loading potatoes");
    loop {
        println!("magicfun(3, 4) = {}", magicfun(3, 4));
        println!("magicfun(5, 8) = {}", magicfun(5, 8));

        let c: u32 = magicfun.get("c");
        println!("{c}");
        magicfun.set("c", c + 1);

        let mut s = String::new();
        print!("Press enter to hot-reload magicfun");
        let _ = stdout().flush();
        stdin().read_line(&mut s).unwrap();
    }
}