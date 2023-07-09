# hot_potato: Hot reloading your rust code

This crate lets you hot reload function bodies and change magic values on the fly,
enabling quick changes without an entire restart and navigation to the current interactible.

Note that this works only in your dev envioronment - not on a shipped build!

It is highly recommended that all potato code should be stripped out of release builds! This crate is meant to facilitate design&development and is not qualified to 
serve soundly in a deplyoed build.

# Quickstart
(see  [test_project](../test_project/src/main.rs) for a rudimentary implementation)

0. Add `hot_potato` to your project<br>
```sh
cargo add hot_potato
```
1. Create the function you want to hot reload and annotate ith with `potato`
```rs
use hot_potato::potato;

/// t: time [0.0;1.0]
#[potato]
fn interpolate(t: f32) -> f32 {
    // linear interpolation for now
    t
}
```
2. Load the functions on startup
```rs
use hot_potato::build_and_reload_potatoes;

fn main() {
    build_and_reload_potatoes().expect("error loading potatoes");
    ...
}
```
3. Create a reload trigger (hotkey, ui widget, or a input like here)
```rs
fn main() {
    // some quick and dirty loop
    loop {
        // make sure this is called at least once before any potato func is called!
        build_and_reload_potatoes().expect("error loading potatoes");

        for i in 0..=5 {
            let t = i as f32 / 5.0;
            println!("{t} -> {}", interpolate(t));
        }

        println!("Press enter to hot-reload");
        // just waits for input and then starts the loop anew...
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
}
```
4. Configure a lib target in your `Cargo.toml`
```toml
[lib]
path = "src/main.rs"
crate-type = ["cdylib"]

[[bin]]
name = "test_project"
path = "src/main.rs"
```
the lib target is for the hot reloading and the bin target is your default run compile target.

5. Run using `cargo run`
Try editing the interpolation function and triggering the reload:+
```rs
/// t: time [0.0;1.0]
#[potato]
fn interpolate(t: f32) -> f32 {
    // quadratic
    t * t
}
```
```rs
/// t: time [0.0;1.0]
#[potato]
fn interpolate(t: f32) -> f32 {
    // ease in-out
    x * x * (3.0 - 2.0 * x)
}
```

# Magic value adjustment
In this scenario we want to show a widget with a certain color but we are not quite happy with it.

Note that you still need an initial `build_and_reload_potatoes`, but no such reload is needed after value adjustment.
```rs
// Pseudo code, look at test_project for some real code

#[potato(r: u8 = 0xFF, g: u8 = 0x00, b: u8 = 0xFF)]
fn show_colored_thing(text: &str, ui: &mut UIContext) -> f32 {
    let mut widget = Widget::new();
    widget.title = Some(text);
    widget.bg_color = Color::from_rgba(r, g, b, 0xFF);
    ui.popup(widget);
}

// Somewhere in an debug ui handler that has an "apply&test" button.
// Also lets just pretend we have 3 debug ui sliders for rgb.
fn on_click_apply(/* pretend ui library args */, ui: &mut UIContext) {
    show_colored_thing.set::<u8>("r", red.get_slider_value());
    show_colored_thing.set::<u8>("g", red.get_slider_value());
    show_colored_thing.set::<u8>("b", red.get_slider_value());

    // This does not need to be called here... 
    // could be anywhere, is just here for testing, 
    // so that we can test the new colors with one click
    show_colored_thing("Test RGB popup", ui);
}
```

## What this can do:
- hot reload almost arbitrary function bodies
- quickly adjust magic values
## What this can't do:
- hot reload function signature changes or arbitrary code
- hot reload trait methods or generics
- make you a sandwich

Warning: Function signature changes and similar wild stuff will result in undefined behavior, msot likely a `STATUS_ACCESS_VIOLATION` or similar crash.