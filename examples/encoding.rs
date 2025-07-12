use scratchback::encoding::Encoding;

fn main() {
    let encoding = Encoding::new();
    let e = encoding.encode("walter is my favorite k-pop singer!").unwrap();
    let d = encoding.decode(&e).unwrap();
    println!("{e:#?}\n{d:#?}");
}
