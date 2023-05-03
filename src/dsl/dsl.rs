use std::str;
use std::vec::Vec;

pub use nom::character::complete::{char, digit1};
use nom::multi::many1;
use nom::sequence::{separated_pair, tuple, delimited};
use nom::{Err, IResult};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, map_res};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicLength {
    Whole,
    Half,
    Fourth,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModdedLength {
    Plain(BasicLength),
    Dotted(BasicLength)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Length {
    Simple(ModdedLength),
    Tied(ModdedLength, ModdedLength),
    Triplet(ModdedLength)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    Hit,
    Rest
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Times(u16);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupOrNote {
    RepeatGroup(Group, Times),
    SingleNote(Note)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group { notes: Vec<GroupOrNote>, length: Length }

static WHOLE : Length = Length::Simple(ModdedLength::Plain(BasicLength::Whole));
static HALF : Length = Length::Simple(ModdedLength::Plain(BasicLength::Half));
static FOURTH : Length = Length::Simple(ModdedLength::Plain(BasicLength::Fourth));
static EIGHTH : Length = Length::Simple(ModdedLength::Plain(BasicLength::Eighth));
static SIXTEENTH : Length = Length::Simple(ModdedLength::Plain(BasicLength::Sixteenth));
static THIRTY_SECOND : Length = Length::Simple(ModdedLength::Plain(BasicLength::ThirtySecond));
static SIXTY_FOURTH : Length = Length::Simple(ModdedLength::Plain(BasicLength::SixtyFourth));

static WHOLE_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::Whole));
static HALF_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::Half));
static FOURTH_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::Fourth));
static EIGHTH_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::Eighth));
static SIXTEENTH_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::Sixteenth));
static THIRTY_SECOND_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::ThirtySecond));
static SIXTY_FOURTH_DOTTED_TRIPLET : Length = Length::Triplet(ModdedLength::Dotted(BasicLength::SixtyFourth));

static WHOLE_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::Whole));
static HALF_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::Half));
static FOURTH_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::Fourth));
static EIGHTH_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::Eighth));
static SIXTEENTH_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::Sixteenth));
static THIRTY_SECOND_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::ThirtySecond));
static SIXTY_FOURTH_TRIPLET : Length = Length::Triplet(ModdedLength::Plain(BasicLength::SixtyFourth));

static HIT : GroupOrNote = GroupOrNote::SingleNote(Note::Hit);
static REST : GroupOrNote = GroupOrNote::SingleNote(Note::Rest);

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
        Ok((r, i)) => {
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

fn group(input: &str) -> IResult<&str, Group> {
    let (rem, (l, n)) = tuple((length, many1(note)))(input)?;
    Ok((rem, Group{ notes: n.iter().map(|x| GroupOrNote::SingleNote(x.clone())).collect(), length: l}))
}

fn delimited_group(input: &str) -> IResult<&str, Group> {
    delimited(char('('), group, char(')'))(input)
}

#[test]
fn parse_length() {
  assert_eq!(length("16"), Ok(("", SIXTEENTH.clone())));
  assert_eq!(length("8+16"), Ok(("", Length::Tied(ModdedLength::Plain(BasicLength::Eighth), ModdedLength::Plain(BasicLength::Sixteenth)))));
  assert_eq!(length("8t"), Ok(("", EIGHTH_TRIPLET.clone())));
  assert_eq!(length("4.t"), Ok(("", FOURTH_DOTTED_TRIPLET.clone())));
}

#[test]
fn parse_group() {
  assert_eq!(group("16x--x-"), Ok(("", Group{notes: vec![HIT.clone(), REST.clone(), REST.clone(), HIT.clone(), REST.clone()], length: SIXTEENTH.clone()})));
  assert_eq!(group("8txxx"), Ok(("", Group { notes: vec![HIT.clone(), HIT.clone(), HIT.clone()], length: EIGHTH_TRIPLET.clone()})));
  assert_eq!(group("16+32x-xx"), Ok(("", Group {notes: vec![HIT.clone(), REST.clone(), HIT.clone(), HIT.clone()], length: Length::Tied(ModdedLength::Plain(BasicLength::Sixteenth), ModdedLength::Plain(BasicLength::ThirtySecond))})))
}

// “x” hit
// “-“ rest
// 16x-- => 16th hit and 16th rests

// - 16x-xx-x-8txxx(3,16+32x-xx)4x-x- => x-xx-x- of 16th, then three hits of 8th triplets, repeat a group of tied 16+32th x-xx three times, then 4th x-x