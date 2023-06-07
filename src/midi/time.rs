extern crate derive_more;
use crate::dsl::dsl::{BasicLength, Group, GroupOrNote, KnownLength, Note, Times, EIGHTH, FOURTH};
use std::cmp::Ordering;

use std;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: BasicLength,
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: BasicLength) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl std::ops::Mul<u8> for TimeSignature {
    type Output = TimeSignature;
    fn mul(self, rhs: u8) -> TimeSignature {
        TimeSignature {
            numerator: self.numerator * rhs as u8,
            denominator: self.denominator,
        }
    }
}

#[test]
fn test_cmp_time_signature() {
    let three_sixteenth = TimeSignature {
        numerator: 3,
        denominator: BasicLength::Sixteenth,
    };
    let four_fourth = TimeSignature {
        numerator: 4,
        denominator: BasicLength::Fourth,
    };
    let two_secondth = TimeSignature {
        numerator: 2,
        denominator: BasicLength::Half,
    };
    assert_eq!(three_sixteenth.cmp(&four_fourth), Ordering::Less);
    // weird, but not worth changing
    // May implement a new type Ord if it needs to be Equal.
    assert_eq!(four_fourth.cmp(&two_secondth), Ordering::Greater);
}

impl KnownLength for TimeSignature {
    fn to_128th(&self) -> u32 {
        self.denominator.to_128th() * self.numerator as u32
    }
}

impl TimeSignature {
    pub fn converges<T: KnownLength>(&self, multiple: Vec<T>) -> Result<u32, String> {
        let bar_len = self.to_128th();
        let result = multiple
            .iter()
            .fold(bar_len, |acc, t| lowest_common_divisor(t.to_128th(), acc));

        let limit = 1000;

        let out = result / bar_len;

        if limit > out {
            Ok(out)
        } else {
            Err("Does not converge".to_string())
        }
    }
}

fn lowest_common_divisor(a: u32, b: u32) -> u32 {
    let mut lcm = u32::max(a, b);

    while lcm % a != 0 || lcm % b != 0 {
        lcm += 1;
    }

    lcm
}

#[test]
fn test_lcm() {
    assert_eq!(lowest_common_divisor(128, 96), 384);
    assert_eq!(lowest_common_divisor(96, 128), 384);
}

#[test]
fn test_converges() {
    let four_fourth = TimeSignature {
        numerator: 4,
        denominator: BasicLength::Fourth,
    };
    let six_fourth = TimeSignature {
        numerator: 6,
        denominator: BasicLength::Fourth,
    };
    let three_fourth = TimeSignature {
        numerator: 3,
        denominator: BasicLength::Fourth,
    };
    let thirteen_eights = Group {
        notes: vec![GroupOrNote::SingleNote(Note::Hit)],
        length: FOURTH.clone(),
        times: Times(12),
    };
    let in_shards_poly = Group {
        notes: vec![
            GroupOrNote::SingleNote(Note::Hit),
            GroupOrNote::SingleNote(Note::Rest),
            GroupOrNote::SingleGroup(thirteen_eights),
        ],
        length: EIGHTH.clone(),
        times: Times(1),
    };
    assert_eq!(three_fourth.converges(vec![four_fourth]), Ok(4));
    assert_eq!(four_fourth.converges(vec![three_fourth]), Ok(3));
    assert_eq!(four_fourth.converges(vec![three_fourth, four_fourth]), Ok(3));
    assert_eq!(four_fourth.converges(vec![three_fourth, six_fourth, four_fourth]), Ok(3));
    assert_eq!(four_fourth.converges(vec![in_shards_poly]), Ok(13));
}
