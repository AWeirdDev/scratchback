//! This module provides encoding and derive macros for cloud variables.
//!
//! Example:
//! ```no_run
//! #[derive(Debug, ScratchObject)]
//! struct Person {
//!     #[id(0)]
//!     name: String,
//! }
//!
//! let person = Person { name: "Walter White".to_string() };
//! let encoded: String = person.sb_encode().unwrap();  // scratchback encode
//! println!("{encoded:?}");
//!
//! let decoded = Person::from_sb_encoded(&encoded).unwrap();
//! println!("{decoded:#?}");
//! ```

pub use scratchback_macros::ScratchObject;

macro_rules! encoding_table {
    ($name:ident, [$(($idx:expr, $ch:expr)),* $(,)?]) => {
        /// An encoding table.
        pub struct $name;

        impl $name {
            pub const TABLE: [char; count_idents!($( $idx ),*)] = [
                $( $ch ),*
            ];

            pub const fn decode(index: usize) -> Option<char> {
                if index < Self::TABLE.len() {
                    Some(Self::TABLE[index])
                } else {
                    None
                }
            }

            pub const fn encode(c: char) -> Option<usize> {
                match c {
                    $( $ch => Some($idx), )*
                    _ => None,
                }
            }
        }
    };
}

// count number of elements
macro_rules! count_idents {
    ($($x:expr),*) => {
        <[()]>::len(&[ $(replace_expr!($x ())),* ])
    };
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

#[rustfmt::skip]
encoding_table!(EncodingTable, [
    (0, '�'), // Placeholder

    (1, '0'), (2, '1'), (3, '2'), (4, '3'), (5, '4'), (6, '5'),
    (7, '6'), (8, '7'), (9, '8'), (10, '9'), (11, 'a'), (12, 'b'),
    (13, 'c'), (14, 'd'), (15, 'e'), (16, 'f'), (17, 'g'), (18, 'h'),
    (19, 'i'), (20, 'j'), (21, 'k'), (22, 'l'), (23, 'm'), (24, 'n'),
    (25, 'o'), (26, 'p'), (27, 'q'), (28, 'r'), (29, 's'), (30, 't'),
    (31, 'u'), (32, 'v'), (33, 'w'), (34, 'x'), (35, 'y'), (36, 'z'),
    (37, 'A'), (38, 'B'), (39, 'C'), (40, 'D'), (41, 'E'), (42, 'F'),
    (43, 'G'), (44, 'H'), (45, 'I'), (46, 'J'), (47, 'K'), (48, 'L'),
    (49, 'M'), (50, 'N'), (51, 'O'), (52, 'P'), (53, 'Q'), (54, 'R'),
    (55, 'S'), (56, 'T'), (57, 'U'), (58, 'V'), (59, 'W'), (60, 'X'),
    (61, 'Y'), (62, 'Z'), (63, '!'), (64, '"'), (65, '#'), (66, '$'),
    (67, '%'), (68, '&'), (69, '\''), (70, '('), (71, ')'), (72, '*'),
    (73, '+'), (74, ','), (75, '-'), (76, '.'), (77, '/'), (78, ':'),
    (79, ';'), (80, '<'), (81, '='), (82, '>'), (83, '?'), (84, '@'),
    (85, '['), (86, '\\'), (87, ']'), (88, '^'), (89, '_'), (90, '`'),
    (91, '{'), (92, '|'), (93, '}'), (94, '~'), (95, ' '), (96, '\n'),

    (97, '•'), // SPLITTER
]);

/// Encoding for `scratchback`.
pub struct Encoding();

impl Encoding {
    pub const SPLITTER: char = '•';
    pub const SPLITTER_STR: &str = "•";

    /// Create a new encoder.
    pub const fn new() -> Self {
        Self()
    }

    pub fn encode(&self, input: &str) -> Option<String> {
        let mut seq = EncodedSequence::new();

        for chr in input.chars() {
            let id = EncodingTable::encode(chr)?;
            seq.add(id);
        }

        Some(seq.get())
    }

    pub fn encode_items(&self, items: &[&str]) -> Option<String> {
        self.encode(&items.join(Self::SPLITTER_STR))
    }

    pub fn decode(&self, numbers: &str) -> Option<String> {
        let mut decoded = String::new();

        if numbers.len() % 2 != 0 {
            return None;
        }

        for idx in (0..numbers.len()).step_by(2) {
            let num = atoi::atoi::<usize>(&numbers[idx..=idx + 1].as_bytes())?;
            decoded.push(EncodingTable::decode(num)?);
        }

        Some(decoded)
    }

    pub fn decode_items(&self, numbers: &str) -> Option<Vec<String>> {
        let mut decoded = Vec::new();
        let mut s = String::new();

        if numbers.len() % 2 != 0 {
            return None;
        }

        for idx in (0..numbers.len()).step_by(2) {
            let num = atoi::atoi::<usize>(&numbers[idx..=idx + 1].as_bytes())?;
            let chr = EncodingTable::decode(num)?;
            if chr == Self::SPLITTER {
                decoded.push(s.drain(..).collect());
            } else {
                s.push(chr);
            }
        }
        if !s.is_empty() {
            decoded.push(s);
        }

        Some(decoded)
    }

    pub fn decode_items_to_array<const N: usize>(&self, numbers: &str) -> Option<[String; N]> {
        let mut vec = self.decode_items(numbers)?;

        if vec.len() != N {
            return None;
        }

        let ptr = vec.as_mut_ptr();
        std::mem::forget(vec);

        Some(unsafe { ptr.cast::<[String; N]>().read() })
    }
}

pub struct EncodedSequence {
    seq: Vec<String>,
}

impl EncodedSequence {
    /// Creates a new encoded sequence.
    fn new() -> Self {
        Self { seq: Vec::new() }
    }

    /// Adds a new item.
    fn add(&mut self, item: usize) {
        let mut bf = itoa::Buffer::new();
        let num = bf
            .format(item)
            .get(..)
            .unwrap();

        self.seq.push(format!("{:0>2}", num))
    }

    fn get(&self) -> String {
        self.seq.join("")
    }
}

pub trait SbStringTo<T> {
    fn sb_string_to(&self) -> Option<T>;
}

impl SbStringTo<String> for String {
    fn sb_string_to(&self) -> Option<String> {
        Some(self.to_owned())
    }
}

macro_rules! impl_atoi_sbstringto {
    ($typ:ty) => {
        impl SbStringTo<$typ> for String {
            fn sb_string_to(&self) -> Option<$typ> {
                atoi::atoi(self.as_bytes())
            }
        }
    };
}

impl_atoi_sbstringto!(u8);
impl_atoi_sbstringto!(u16);
impl_atoi_sbstringto!(u32);
impl_atoi_sbstringto!(u64);
impl_atoi_sbstringto!(i8);
impl_atoi_sbstringto!(i16);
impl_atoi_sbstringto!(i32);
impl_atoi_sbstringto!(i64);

pub trait SbToString {
    fn sb_to_string(&self) -> String;
}

impl SbToString for String {
    fn sb_to_string(&self) -> String {
        self.clone()
    }
}

macro_rules! impl_atoi_sbtostring {
    ($typ:ty) => {
        impl SbToString for $typ {
            fn sb_to_string(&self) -> String {
                let mut buf = itoa::Buffer::new();
                buf.format(*self).to_string()
            }
        }
    };
}

impl_atoi_sbtostring!(u8);
impl_atoi_sbtostring!(u16);
impl_atoi_sbtostring!(u32);
impl_atoi_sbtostring!(u64);
impl_atoi_sbtostring!(i8);
impl_atoi_sbtostring!(i16);
impl_atoi_sbtostring!(i32);
impl_atoi_sbtostring!(i64);
