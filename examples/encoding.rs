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
    let stuff = Stuff::A(Person { name: "asdf".to_string() });
    let encoded = stuff.sb_encode().unwrap();
    println!("{encoded:#?}");
    let a = Stuff::from_sb_encoded(&encoded);
    println!("{a:#?}");

    let a = Person::from_sb_encoded("0111291416");
    println!("{a:#?}");
}
