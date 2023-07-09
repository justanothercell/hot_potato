use std::io::{stdout, stdin, Write};

use hot_potato::{potato, build_and_reload_potatoes};

#[potato(magic: u32 = 42, sparkles: u32 = 69)]
fn magicfun(a: u32, b: u32) -> u32 {
    a + b + (magic + sparkles)
}

#[allow(unused)]
fn main() {
    loop {
        build_and_reload_potatoes().expect("error loading potatoes");
        println!("reloaded!");
        println!("magicfun(3, 4) = {}", magicfun(3, 4));
        println!("magicfun(5, 8) = {}", magicfun(5, 8));

        let v: u32 = magicfun.get("magic");
        magicfun.set("magic", v + 1);

        let mut s = String::new();
        print!("Press enter to hot-reload magicfun");
        let _ = stdout().flush();
        stdin().read_line(&mut s).unwrap();
    }
}