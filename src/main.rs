use bevy::prelude::*;

mod ship;
mod state;
fn main() {
    App::new().add_systems(Update, hello_world).run();
}

fn hello_world() {
    println!("hello_world!");
}
