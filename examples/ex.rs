use generational_arena::Arena;

fn main() {
    let mut a = Arena::new();
    let z = a.typed_insert(10);
    a.typed_remove(z);
    let z = a.typed_insert(10);
    a.typed_remove(z);
    let z = a.typed_insert(10);
    a.typed_remove(z);
    let z = a.typed_insert(10);
    println!("index {:?}", z);
}
