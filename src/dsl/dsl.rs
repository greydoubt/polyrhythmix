use std::num::ParseIntError;
use std::str::{self, FromStr};
use std::vec::Vec;
use std::ops::{Add};

pub use nom::character::complete::{char, digit1};
use nom::multi::many1;
use nom::sequence::{separated_pair, tuple, delimited};
use nom::{Err, IResult};
use nom::branch::alt;

use nom::combinator::{map, map_res};


/// Allows measurement in 128th notes.
pub trait KnownLength {
    fn to_128th(&self) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BasicLength {
    Whole,
    Half,
    Fourth,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth
}

impl FromStr for BasicLength {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Result<u16, ParseIntError> = s.parse();
        match result {
            Ok(n) => Self::from_num(n),
            Result::Err(e) => panic!("{}", e)
        }
    }
}

impl KnownLength for BasicLength {
    fn to_128th(&self) -> u32 {
        match self {
            BasicLength::Whole => 128,
            BasicLength::Half => 64,
            BasicLength::Fourth => 32,
            BasicLength::Eighth => 16,
            BasicLength::Sixteenth => 8,
            BasicLength::ThirtySecond => 4,
            BasicLength::SixtyFourth => 2,
        }
    }
}

impl BasicLength {
    /// Unsafe method, so it should only be used from `add`.
    fn from_num(n: u16) -> Result<Self, String> {
        match n {
            1 => Ok(BasicLength::SixtyFourth),
            2 => Ok(BasicLength::ThirtySecond),
            4 => Ok(BasicLength::Sixteenth),
            8 => Ok(BasicLength::Eighth),
            16 => Ok(BasicLength::Fourth),
            32 => Ok(BasicLength::Half),
            64 => Ok(BasicLength::Whole),
            e => Err(format!("{} is not a num BasicLength can be constructed from", e))
        }
    }
}

impl Add<BasicLength> for BasicLength {
    type Output = Length;

    fn add(self, rhs: BasicLength) -> Length {
        let f = |x| match x {
            BasicLength::Whole => 64,
            BasicLength::Half => 32,
            BasicLength::Fourth => 16,
            BasicLength::Eighth => 8,
            BasicLength::Sixteenth => 4,
            BasicLength::ThirtySecond => 2,
            BasicLength::SixtyFourth => 1,
        };
        if self == rhs && self != BasicLength::Whole {
            Length::Simple(ModdedLength::Plain(BasicLength::from_num(f(self) * 2).unwrap()))
        } else {
            let n1 : u16 = f(self);
            let n2 = f(rhs);
            let total = n1 + n2;
            
            if total > 64 {
                Length::Tied(ModdedLength::Plain(BasicLength::Whole), ModdedLength::Plain(BasicLength::from_num(total - 64).unwrap()))
            } else if total > 32 {
                Length::Tied(ModdedLength::Plain(BasicLength::Half), ModdedLength::Plain(BasicLength::from_num(total - 32).unwrap()))
            } else if total > 16 {
                Length::Tied(ModdedLength::Plain(BasicLength::Fourth), ModdedLength::Plain(BasicLength::from_num(total - 16).unwrap()))
            } else if total > 8 {
                Length::Tied(ModdedLength::Plain(BasicLength::Eighth), ModdedLength::Plain(BasicLength::from_num(total - 8).unwrap()))
            } else if total > 4 {
                Length::Tied(ModdedLength::Plain(BasicLength::Fourth), ModdedLength::Plain(BasicLength::from_num(total - 4).unwrap()))
            } else {
                Length::Tied(ModdedLength::Plain(BasicLength::Half), ModdedLength::Plain(BasicLength::from_num(total - 2).unwrap()))
            }
        }
    }
}

#[test]
fn test_add_basic_length() {
    assert_eq!(BasicLength::Half + BasicLength::Half, Length::Simple(ModdedLength::Plain(BasicLength::Whole)));
    assert_eq!(BasicLength::Whole + BasicLength::Whole, Length::Tied(ModdedLength::Plain(BasicLength::Whole), ModdedLength::Plain(BasicLength::Whole)));
    assert_eq!(BasicLength::Half + BasicLength::SixtyFourth, Length::Tied(ModdedLength::Plain(BasicLength::Half), ModdedLength::Plain(BasicLength::SixtyFourth)));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModdedLength {
    Plain(BasicLength),
    Dotted(BasicLength)
}

impl KnownLength for ModdedLength {
    fn to_128th(&self) -> u32 {
        match self {
            ModdedLength::Plain(bl) => bl.to_128th(),
            ModdedLength::Dotted(bl) => {
                let l = bl.to_128th();
                l + l /2
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    Simple(ModdedLength),
    Tied(ModdedLength, ModdedLength),
    Triplet(ModdedLength)
}

impl KnownLength for Length {
    fn to_128th(&self) -> u32 {
        match self {
            Length::Simple(ml) => ml.to_128th(),
            Length::Tied(ml1, ml2) => ml1.to_128th() + ml2.to_128th(),
            Length::Triplet(ml) => ml.to_128th() * 2 / 3
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    Hit,
    Rest
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Times(pub u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupOrNote {
    SingleGroup(Group),
    SingleNote(Note)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub notes: Vec<GroupOrNote>,
    pub length: Length,
    pub times: Times
}

impl KnownLength for Group {
    fn to_128th(&self) -> u32 {
        let mut acc = 0;
        let note_length = self.length.to_128th();
        for group in self.notes.iter() {
            match group {
                GroupOrNote::SingleGroup(subgroup) => { acc += subgroup.to_128th(); },
                GroupOrNote::SingleNote(_) => { acc += note_length; },
            }
        };
        acc * self.times.0 as u32
    }
}

impl std::ops::Deref for Group {
    type Target = Vec<GroupOrNote>;

    fn deref(&self) -> &Self::Target {
        &self.notes
    }
}

#[allow(dead_code)]
pub static WHOLE : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::Whole));
#[allow(dead_code)]
pub static HALF : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::Half));
#[allow(dead_code)]
pub static FOURTH : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::Fourth));
#[allow(dead_code)]
pub static EIGHTH : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::Eighth));
#[allow(dead_code)]
pub static SIXTEENTH : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::Sixteenth));
#[allow(dead_code)]
pub static THIRTY_SECOND : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::ThirtySecond));
#[allow(dead_code)]
pub static SIXTY_FOURTH : &Length = &Length::Simple(ModdedLength::Plain(BasicLength::SixtyFourth));

#[allow(dead_code)]
pub static WHOLE_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::Whole));
#[allow(dead_code)]
pub static HALF_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::Half));
#[allow(dead_code)]
pub static FOURTH_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::Fourth));
#[allow(dead_code)]
pub static EIGHTH_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::Eighth));
#[allow(dead_code)]
pub static SIXTEENTH_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::Sixteenth));
#[allow(dead_code)]
pub static THIRTY_SECOND_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::ThirtySecond));
#[allow(dead_code)]
pub static SIXTY_FOURTH_DOTTED_TRIPLET : &Length = &Length::Triplet(ModdedLength::Dotted(BasicLength::SixtyFourth));

#[allow(dead_code)]
pub static WHOLE_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::Whole));
#[allow(dead_code)]
pub static HALF_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::Half));
#[allow(dead_code)]
pub static FOURTH_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::Fourth));
#[allow(dead_code)]
pub static EIGHTH_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::Eighth));
#[allow(dead_code)]
pub static SIXTEENTH_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::Sixteenth));
#[allow(dead_code)]
pub static THIRTY_SECOND_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::ThirtySecond));
#[allow(dead_code)]
pub static SIXTY_FOURTH_TRIPLET : &Length = &Length::Triplet(ModdedLength::Plain(BasicLength::SixtyFourth));

#[allow(dead_code)]
pub static HIT : GroupOrNote = GroupOrNote::SingleNote(Note::Hit);
#[allow(dead_code)]
pub static REST : GroupOrNote = GroupOrNote::SingleNote(Note::Rest);

#[allow(dead_code)]
pub static ONCE : &Times = &Times(1);
#[allow(dead_code)]
pub static TWICE: &Times = &Times(2);
#[allow(dead_code)]
pub static THRICE : &Times = &Times(3);


fn hit(input: &str) -> IResult<&str, Note> {
    map(char('x'), |_| { Note::Hit })(input)
}

fn rest(input: &str) -> IResult<&str, Note> {
    map(char('-'), |_| { Note::Rest })(input)
}

fn note(input: &str) -> IResult<&str, Note> {
    alt((hit, rest))(input)
}

fn length_basic(input: &str) -> IResult<&str, BasicLength> {
    match map_res(digit1, str::parse)(input) {
        Ok((r,1)) => Ok((r, BasicLength::Whole)),
        Ok((r,2)) => Ok((r, BasicLength::Half)),
        Ok((r,4)) => Ok((r, BasicLength::Fourth)),
        Ok((r,8)) => Ok((r, BasicLength::Eighth)),
        Ok((r,16)) => Ok((r, BasicLength::Sixteenth)),
        Ok((r,32)) => Ok((r, BasicLength::ThirtySecond)),
        Ok((r, 64)) => Ok((r, BasicLength::SixtyFourth)),
        Ok((r, _)) => {
            Err(Err::Error(nom::error::make_error(r, nom::error::ErrorKind::Fail)))
        },
        Err(e) => Err(e)
    }
}

fn dotted_length(input: &str) -> IResult<&str, ModdedLength> {
    map(tuple((length_basic, char('.'))), |(l, _)| { ModdedLength::Dotted(l)})(input)
}

fn modded_length(input: &str) -> IResult<&str, ModdedLength> {
    alt((dotted_length, map(length_basic, |x| {ModdedLength::Plain(x)})))(input)
}

fn triplet_length(input: &str) -> IResult<&str, Length> {
    map(tuple((modded_length, char('t'))), |(l, _)| { Length::Triplet(l)})(input)
}

fn tied_length(input: &str) -> IResult<&str, Length> {
    map(separated_pair(modded_length, char('+'), modded_length), |(x, y)| { Length::Tied(x,y)})(input)
}

fn length(input: &str) -> IResult<&str, Length> {
    alt((triplet_length, tied_length, map(modded_length, |x| { Length::Simple(x) })))(input)
}

fn times(input: &str) -> IResult<&str, Times> {
    map(map_res(digit1, str::parse), |x| { Times(x) } )(input)
}

fn group(input: &str) -> IResult<&str, Group> {
    let repeated = map(tuple((times, char(','), length, many1(note))), |(t, _, l, n)| { (t, l, n)} );
    let single = map(tuple((length, many1(note))), |(l, vn)| { (Times(1), l, vn) } );
    let (rem, (t, l, n)) = alt((repeated, single))(input)?;
    Ok((rem, Group{ notes: n.iter().map(|x| GroupOrNote::SingleNote(x.clone())).collect(), length: l, times: t}))
}

fn delimited_group(input: &str) -> IResult<&str, Group> {
    delimited(char('('), group, char(')'))(input)
}

pub fn group_or_delimited_group(input: &str) -> IResult<&str, Group> {
  alt((delimited_group, group))(input)
}

pub fn groups(input: &str) -> IResult<&str, Vec<Group>> {
    many1(group_or_delimited_group)(input)
}

#[test]
fn parse_length() {
  assert_eq!(length("16"), Ok(("", *SIXTEENTH)));
  assert_eq!(length("8+16"), Ok(("", Length::Tied(ModdedLength::Plain(BasicLength::Eighth), ModdedLength::Plain(BasicLength::Sixteenth)))));
  assert_eq!(length("8t"), Ok(("", *EIGHTH_TRIPLET)));
  assert_eq!(length("4.t"), Ok(("", *FOURTH_DOTTED_TRIPLET)));
}

#[test]
fn parse_group() {
  assert_eq!(group("16x--x-"), Ok(("", Group { times: *ONCE, notes: vec![HIT.clone(), REST.clone(), REST.clone(), HIT.clone(), REST.clone()], length: *SIXTEENTH})));
  assert_eq!(group("8txxx"), Ok(("", Group { times: *ONCE, notes: vec![HIT.clone(), HIT.clone(), HIT.clone()], length: *EIGHTH_TRIPLET})));
  assert_eq!(group("16+32x-xx"), Ok(("", Group { times: *ONCE, notes: vec![HIT.clone(), REST.clone(), HIT.clone(), HIT.clone()], length: Length::Tied(ModdedLength::Plain(BasicLength::Sixteenth), ModdedLength::Plain(BasicLength::ThirtySecond))})));
  assert_eq!(group("3,16xx"), Ok(("", Group { times: *THRICE, length: *SIXTEENTH, notes: vec![HIT.clone(), HIT.clone()] })));
}

#[test]
fn parse_delimited_group() {
    assert_eq!(delimited_group("(3,16x--x-)"), Ok(("", Group { times: *THRICE, notes: vec![HIT.clone(), REST.clone(), REST.clone(), HIT.clone(), REST.clone()], length: *SIXTEENTH})));
}

#[test]
fn parse_group_or_delimited_group() {
    assert_eq!(group_or_delimited_group("(3,16x--x-)"), Ok(("", Group { times: *THRICE, notes: vec![HIT.clone(), REST.clone(), REST.clone(), HIT.clone(), REST.clone()], length: *SIXTEENTH})));
    assert_eq!(group_or_delimited_group("16x--x-"), Ok(("", Group { times: *ONCE, notes: vec![HIT.clone(), REST.clone(), REST.clone(), HIT.clone(), REST.clone()], length: *SIXTEENTH})));
}

// “x” hit
// “-“ rest
// 16x-- => 16th hit and 16th rests

// - 16x-xx-x-8txxx(3,16+32x-xx)4x-x- => x-xx-x- of 16th, then three hits of 8th triplets, repeat a group of tied 16+32th x-xx three times, then 4th x-x