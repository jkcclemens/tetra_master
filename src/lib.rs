extern crate rand;

use rand::{thread_rng, Rng};

use std::cmp::{min, max};
use std::mem::{self, uninitialized};
use std::cell::Cell;

const STAT_RANGES: &'static [[u8; 2]] = &[
  [0, 15],
  [16, 31],
  [32, 47],
  [48, 63],
  [64, 79],
  [80, 95],
  [96, 111],
  [112, 127],
  [128, 143],
  [144, 159],
  [160, 175],
  [176, 191],
  [192, 207],
  [208, 223],
  [224, 239],
  [240, 255]
];

fn stat(level: u8) -> Option<u8> {
  if level > 0x0F {
    return None;
  }
  let range = STAT_RANGES[level as usize];
  Some(thread_rng().gen_range(range[0] as u16, range[1] as u16 + 1) as u8)
}

pub struct TetraMaster;

impl TetraMaster {
  pub fn explain_battle(attacker: &Card, defender: &Card) -> BattleResult {
    println!("Attacker:");
    println!("{:#?}", attacker);
    println!("Defender:");
    println!("{:#?}", defender);
    let (kind, attack_stat, defense_stat) = match attacker.class {
      Class::Physical => ("physical", "power", "physical defense"),
      Class::Magical => ("magical", "power", "magical defense"),
      Class::Flexible => ("flexible", "power", "lowest stat"),
      Class::Assault => ("assault", "highest stat", "lowest stat")
    };
    println!("The attacker is a {} card, so it will use its {} level to attack the defender's {}.",
      kind,
      attack_stat,
      defense_stat);
    let attacker_level = attacker.offense_level();
    let attacker_range = STAT_RANGES[attacker_level as usize];
    println!("The attacker's level is {}. That means it will roll between {} and {} to determine its max attack score.",
      attacker_level,
      attacker_range[0],
      attacker_range[1]);
    let max_attacker_score = stat(attacker_level).unwrap();
    println!("The attacker's max score is {}.", max_attacker_score);
    let defender_level = attacker.defense_level(defender);
    let defender_range = STAT_RANGES[defender_level as usize];
    println!("The defender's level is {}. That means it will roll between {} and {} to determine its max defense score.",
      defender_level,
      defender_range[0],
      defender_range[1]);
    let max_defender_score = stat(defender_level).unwrap();
    println!("The defender's max score is {}.", max_defender_score);
    let attacker_score = thread_rng().gen_range(0, max_attacker_score as u16 + 1) as u8;
    println!("The attacker now rolls a random number between 0 and its max attack score: {}.", attacker_score);
    let defender_score = thread_rng().gen_range(0, max_defender_score as u16 + 1) as u8;
    println!("The defender now rolls a random number between 0 and its max defense score: {}.", defender_score);
    let final_attack_score = max_attacker_score - attacker_score;
    println!("The attacker now subtracts its score from its max score ({} - {}): {}.",
      max_attacker_score,
      attacker_score,
      final_attack_score);
    let final_defense_score = max_defender_score - defender_score;
    println!("The defender now subtracts its score from its max score ({} - {}): {}.",
      max_defender_score,
      defender_score,
      final_defense_score);
    println!("The card with the highest final score wins.");
    if final_attack_score == final_defense_score {
      println!("The scores were equal, so the battle is a draw.");
      BattleResult::Draw
    } else if final_attack_score > final_defense_score {
      println!("The attacker's score was higher than the defender's score, so the attacker wins.");
      BattleResult::Attacker
    } else {
      println!("The defender's score was higher than the attacker's score, so the defender wins.");
      BattleResult::Defender
    }
  }

  pub fn battle(attacker: &Card, defender: &Card) -> BattleResult {
    let attacker_power = stat(attacker.offense_level()).expect("Invalid card");
    let defender_defense = stat(attacker.defense_level(defender)).expect("Invalid card");
    let attack_score = thread_rng().gen_range(0, attacker_power as u16 + 1) as u8;
    let defense_score = thread_rng().gen_range(0, defender_defense as u16 + 1) as u8;
    let final_attack = attacker_power - attack_score;
    let final_defense = defender_defense - defense_score;
    if final_attack == final_defense {
      BattleResult::Draw
    } else if final_attack > final_defense {
      BattleResult::Attacker
    } else {
      BattleResult::Defender
    }
  }

  pub fn parse_card(values: &str) -> Option<Card> {
    let chars: Vec<char> = values.chars().collect();
    if chars.len() != 4 {
      return None;
    }
    let power = match u8::from_str_radix(&chars[0].to_string(), 16) {
      Ok(x) => x,
      Err(_) => return None
    };
    let class = match chars[1].to_lowercase().next() {
      Some('p') => Class::Physical,
      Some('m') => Class::Magical,
      Some('x') => Class::Flexible,
      Some('a') => Class::Assault,
      _ => return None
    };
    let phys_def = match u8::from_str_radix(&chars[2].to_string(), 16) {
      Ok(x) => x,
      Err(_) => return None
    };
    let mag_def = match u8::from_str_radix(&chars[3].to_string(), 16) {
      Ok(x) => x,
      Err(_) => return None
    };
    Some(Card::new(power, class, phys_def, mag_def))
  }
}

#[derive(Debug)]
pub enum Space {
  Block,
  Card(PlacedCard),
  Empty
}

impl Space {
  pub fn is_block(&self) -> bool {
    match *self {
      Space::Block => true,
      _ => false
    }
  }

  pub fn is_empty(&self) -> bool {
    match *self {
      Space::Empty => true,
      _ => false
    }
  }

  pub fn is_card(&self) -> bool {
    match *self {
      Space::Card(_) => true,
      _ => false
    }
  }
}

impl ToString for Space {
  fn to_string(&self) -> String {
    match *self {
      Space::Block => "XXXX".to_string(),
      Space::Empty => "    ".to_string(),
      Space::Card(ref card) => card.to_string()
    }
  }
}

#[derive(Debug)]
pub struct Board {
  pub spaces: [[Space; 4]; 4]
}

impl Board {
  pub fn generate() -> Self {
    let mut slice: [[Space; 4]; 4] = unsafe { [uninitialized(), uninitialized(), uninitialized(), uninitialized()] };
    let mut blocks = 0;
    for i in 0..4 {
      slice[i] = Board::generate_row(&mut blocks);
    }
    Board {
      spaces: slice
    }
  }

  fn generate_row(blocks: &mut u8) -> [Space; 4] {
    let mut slice: [Space; 4] = unsafe { [uninitialized(), uninitialized(), uninitialized(), uninitialized()] };
    for i in 0..4 {
      slice[i] = if *blocks < 6 && thread_rng().gen_weighted_bool(4) {
        *blocks += 1;
        Space::Block
      } else {
        Space::Empty
      };
    }
    slice
  }

  pub fn add_card(&mut self, row: usize, column: usize, card: OwnedCard) {
    self.spaces[row - 1][column - 1] = Space::Card(PlacedCard::new(card, row, column));
  }

  pub fn remove_card(&mut self, row: usize, column: usize) -> Option<OwnedCard> {
    let mut space = Space::Empty;
    mem::swap(&mut self.spaces[row - 1][column - 1], &mut space);
    match space {
      Space::Card(c) => Some(c.card),
      _ => None
    }
  }

  pub fn space(&self, row: usize, column: usize) -> &Space {
    &self.spaces[row - 1][column - 1]
  }

  pub fn space_mut(&mut self, row: usize, column: usize) -> &mut Space {
    &mut self.spaces[row - 1][column - 1]
  }

  /// Finds neighboring cards of the card at the given location.
  ///
  /// In relation to the given location, the order of the cards returned is West, East, North,
  /// Northwest, Northeast, South, Southwest, Southeast.
  ///
  /// If the location isn't a card, an empty vector is returned.
  pub fn neighbors(&self, row: usize, column: usize) -> Vec<Option<&PlacedCard>> {
    if !self.space(row, column).is_card() {
      return Vec::new();
    }
    let mut cards = Vec::new();
    for r in &[row, row - 1, row + 1] {
      for c in &[column, column - 1, column + 1] {
        if *r == row && *c == column {
          continue;
        }
        if *r > 4 || *c > 4 || *r < 1 || *c < 1 {
          cards.push(None);
          continue;
        }
        let card = match *self.space(*r, *c) {
          Space::Card(ref c) => Some(c),
          _ => None
        };
        cards.push(card);
      }
    }
    cards
  }

  fn do_combo(&self, winner: &PlacedCard, loser: &PlacedCard) {
    let combos: Vec<&PlacedCard> = self.neighbors(loser.row, loser.column)
      .into_iter()
      .enumerate()
      .filter(|&(_, x)| x.is_some())
      .map(|(i, x)| (i, x.unwrap()))
      .filter(|&(_, x)| x.color.get() != winner.color.get())
      .map(|(i, x)| (loser.arrows.relation_from(i.into(), &x.arrows), x))
      .filter(|&(ref r, _)| *r != ArrowRelation::Ignore)
      .map(|(_, x)| x)
      .collect();
    for combo in combos {
      combo.color.set(winner.color.get());
    }
  }

  pub fn run_battles(&self, row: usize, col: usize) {
    let card = match *self.space(row, col) {
      Space::Card(ref c) => c,
      _ => return
    };
    let relations: Vec<(ArrowRelation, &PlacedCard)> = self.neighbors(row, col)
      .into_iter()
      .enumerate()
      .filter(|&(_, x)| x.is_some())
      .map(|(i, x)| (i, x.unwrap()))
      .filter(|&(_, x)| x.color.get() != card.color.get())
      .map(|(i, x)| (card.arrows.relation_from(i.into(), &x.arrows), x))
      .collect();
    let battles: Vec<&PlacedCard> = relations.iter()
      .filter(|&&(ref rel, _)| *rel == ArrowRelation::Battle)
      .map(|&(_, c)| c)
      .collect();
    let mut lost_any = false;
    for defender in &battles {
      match TetraMaster::battle(card, defender) {
        BattleResult::Attacker => {
          defender.color.set(card.color.get());
          self.do_combo(card, defender);
        },
        BattleResult::Defender => {
          card.color.set(defender.color.get());
          self.do_combo(defender, card);
          lost_any = true;
          break;
        },
        BattleResult::Draw => {
          self.run_battles(row, col);
          return;
        }
      }
    }
    if !lost_any {
      let takes = relations.iter()
        .filter(|&&(ref rel, _)| *rel == ArrowRelation::Take)
        .map(|&(_, c)| c);
      for take in takes {
        take.color.set(card.color.get());
      }
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BattleResult {
  Attacker,
  Defender,
  Draw
}

#[derive(Debug)]
pub struct Card {
  pub power: u8,
  pub class: Class,
  pub physical_defense: u8,
  pub magical_defense: u8,
  pub arrows: Arrows
}

impl Card {
  pub fn new(power: u8, class: Class, phys_def: u8, mag_def: u8) -> Self {
    Card {
      power: power,
      class: class,
      physical_defense: phys_def,
      magical_defense: mag_def,
      arrows: Arrows::default()
    }
  }

  pub fn with_arrows(power: u8, class: Class, phys_def: u8, mag_def: u8, arrows: Arrows) -> Self {
    let mut card = Card::new(power, class, phys_def, mag_def);
    card.arrows = arrows;
    card
  }

  /// Gets this card's offense level.
  pub fn offense_level(&self) -> u8 {
    match self.class {
      Class::Physical | Class::Magical | Class::Flexible => self.power,
      Class::Assault => max(max(self.physical_defense, self.magical_defense), self.power)
    }
  }

  /// Gets the defense level this card will use to attack with, given the defending card.
  pub fn defense_level(&self, card: &Card) -> u8 {
    match self.class {
      Class::Physical => card.physical_defense,
      Class::Magical => card.magical_defense,
      Class::Flexible => min(card.physical_defense, card.magical_defense),
      Class::Assault => min(min(card.physical_defense, card.magical_defense), card.power)
    }
  }
}

impl ToString for Card {
  fn to_string(&self) -> String {
    format!("{}{}{}{}",
      format!("{:X}", self.power),
      self.class.as_char(),
      format!("{:X}", self.physical_defense),
      format!("{:X}", self.magical_defense))
  }
}

#[derive(Debug)]
pub enum Direction {
  West,
  East,
  North,
  Northwest,
  Northeast,
  South,
  Southwest,
  Southeast
}

impl From<usize> for Direction {
  fn from(i: usize) -> Direction {
    match i {
      0 => Direction::West,
      1 => Direction::East,
      2 => Direction::North,
      3 => Direction::Northwest,
      4 => Direction::Northeast,
      5 => Direction::South,
      6 => Direction::Southwest,
      7 => Direction::Southeast,
      _ => panic!("Invalid direction")
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArrowRelation {
  Ignore,
  Take,
  Battle
}

#[derive(Debug, Default)]
pub struct Arrows {
  pub flags: u8
}

impl Arrows {
  pub fn from_flags(flags: u8) -> Self {
    Arrows {
      flags: flags
    }
  }

  pub fn relation_from(&self, direction: Direction, other: &Arrows) -> ArrowRelation {
    let (attack, defend) = match direction {
      Direction::North => (self.north(), other.south()),
      Direction::Northeast => (self.northeast(), other.southwest()),
      Direction::East => (self.east(), other.west()),
      Direction::Southeast => (self.southeast(), other.northwest()),
      Direction::South => (self.south(), other.north()),
      Direction::Southwest => (self.southwest(), other.northeast()),
      Direction::West => (self.west(), other.east()),
      Direction::Northwest => (self.northwest(), other.southeast())
    };
    if attack && defend {
      ArrowRelation::Battle
    } else if attack {
      ArrowRelation::Take
    } else {
      ArrowRelation::Ignore
    }
  }

  pub fn north(&self) -> bool {
    (self.flags & (1 << 7)) > 0
  }

  pub fn set_north(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 7;
    } else {
      self.flags &= !(1 << 7);
    }
  }

  pub fn northeast(&self) -> bool {
    (self.flags & (1 << 6)) > 0
  }

  pub fn set_northeast(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 6;
    } else {
      self.flags &= !(1 << 6);
    }
  }

  pub fn east(&self) -> bool {
    (self.flags & (1 << 5)) > 0
  }

  pub fn set_east(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 5;
    } else {
      self.flags &= !(1 << 5);
    }
  }

  pub fn southeast(&self) -> bool {
    (self.flags & (1 << 4)) > 0
  }

  pub fn set_southeast(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 4;
    } else {
      self.flags &= !(1 << 4);
    }
  }

  pub fn south(&self) -> bool {
    (self.flags & (1 << 3)) > 0
  }

  pub fn set_south(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 3;
    } else {
      self.flags &= !(1 << 3);
    }
  }

  pub fn southwest(&self) -> bool {
    (self.flags & (1 << 2)) > 0
  }

  pub fn set_southwest(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 2;
    } else {
      self.flags &= !(1 << 2);
    }
  }

  pub fn west(&self) -> bool {
    (self.flags & (1 << 1)) > 0
  }

  pub fn set_west(&mut self, status: bool) {
    if status {
      self.flags |= 1 << 1;
    } else {
      self.flags &= !(1 << 1);
    }
  }

  pub fn northwest(&self) -> bool {
    (self.flags & 1) > 0
  }

  pub fn set_northwest(&mut self, status: bool) {
    if status {
      self.flags |= 1;
    } else {
      self.flags &= !1;
    }
  }
}

#[derive(Debug)]
pub enum Class {
  Physical,
  Magical,
  Flexible,
  Assault
}

impl Class {
  pub fn as_char(&self) -> char {
    match *self {
      Class::Physical => 'P',
      Class::Magical => 'M',
      Class::Flexible => 'X',
      Class::Assault => 'A'
    }
  }
}

#[derive(Debug)]
pub struct OwnedCard {
  pub card: Card,
  pub color: Cell<Color>
}

impl std::ops::Deref for OwnedCard {
  type Target = Card;

  fn deref(&self) -> &Self::Target {
    &self.card
  }
}

impl OwnedCard {
  pub fn new(card: Card, color: Color) -> Self {
    OwnedCard {
      card: card,
      color: Cell::new(color)
    }
  }

  pub fn blue(card: Card) -> Self {
    Self::new(card, Color::Blue)
  }

  pub fn red(card: Card) -> Self {
    Self::new(card, Color::Red)
  }

  pub fn into_inner(self) -> Card {
    self.card
  }
}

#[derive(Debug)]
pub struct PlacedCard {
  pub card: OwnedCard,
  pub row: usize,
  pub column: usize
}

impl std::ops::Deref for PlacedCard {
  type Target = OwnedCard;

  fn deref(&self) -> &Self::Target {
    &self.card
  }
}

impl PlacedCard {
  pub fn new(card: OwnedCard, row: usize, column: usize) -> Self {
    PlacedCard {
      card: card,
      row: row,
      column: column
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
  Blue,
  Red
}
