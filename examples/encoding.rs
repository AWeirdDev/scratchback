use scratchback::encoding::ScratchObject;

#[derive(Debug, ScratchObject)]
struct Player {
    #[id(0)]
    name: String,

    #[id(1)]
    happy: bool,
}

fn main() {
    let player = Player {
        name: "Walt".to_string(),
        happy: true,
    };
    let encoded = player.sb_encode().unwrap();

    println!("{:?}", Player::from_sb_encoded(&encoded));
}
