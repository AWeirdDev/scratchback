use scratchback::encoding::{ Encoding, ScratchObject };

#[derive(Debug, ScratchObject)]
struct Person {
    #[id(0)]
    name: String,
}

#[derive(Debug, ScratchObject)]
enum Stuff {
    #[id(10)] A(Person),
}

fn main() {
    
}
