use std::io::{stdout, stdin, Write};

use hot_potato::{potato, build_and_reload_potatoes};

#[potato]
fn magicfun(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
fn testy() {
    println!("success!")
}

fn main() {
    loop {
        build_and_reload_potatoes().expect("error loading potatoes");
        println!("reloaded!");
        println!("magicfun(3, 4) = {}", magicfun(3, 4));
        println!("magicfun(5, 8) = {}", magicfun(5, 8));

        let mut s = String::new();
        print!("Press enter to hot-reload magicfun");
        let _ = stdout().flush();
        stdin().read_line(&mut s).unwrap();
    }
    
}