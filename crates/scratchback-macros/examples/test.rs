use scratchback::encoding::Encoding;
use scratchback_macros::ScratchObject;

#[derive(Debug, ScratchObject)]
struct Stuff {
    #[id(0)]
    number: u32,

    #[id(1)]
    seat: u32,
}

fn main() {
    let stuff = Stuff {
        number: 45,
        seat: 10000000,
    };
    let encoded = stuff.sb_encode().unwrap();
    println!("{encoded:#?}");
    let a = Stuff::from_sb_encoded(&encoded).unwrap();
    println!("{a:#?}");
}
