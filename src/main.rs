use bevy::prelude::*;

fn main() {
    App::new()
        .add_startup_system(add_people)
        .add_system(hello_world)
        .add_system(greet_people)
        .run();
}

fn hello_world() {
    println!("Howdy!");
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn().insert(Person).insert(Name("Sam Smith".to_string()));
    commands.spawn().insert(Person).insert(Name("Jon Jones".to_string()));
    commands.spawn().insert(Person).insert(Name("Bob Brown".to_string()));
}

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in query.iter() {
        println!("hello {}", name.0);
    }
}
