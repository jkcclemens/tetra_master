extern crate tetra_master;

use tetra_master::{TetraMaster, BattleResult};

use std::env::args;

fn main() {
  let args: Vec<String> = args().skip(1).collect();
  if args.len() < 2 {
    println!("Usage: battle card_1 card_2 (explain)");
    println!("Specify two cards (e.g. 1M23 2P34). Attacker first, defender second.");
    return;
  }
  let explain = args.len() > 2 && args[2].to_lowercase() == "explain";
  let attacker = match TetraMaster::parse_card(&args[0]) {
    Some(c) => c,
    None => {
      println!("First card (attacker) was invalid.");
      return;
    }
  };
  let defender = match TetraMaster::parse_card(&args[1]) {
    Some(c) => c,
    None => {
      println!("Second card (defender) was invalid.");
      return;
    }
  };
  let result = if explain {
    TetraMaster::explain_battle(&attacker, &defender)
  } else {
    TetraMaster::battle(&attacker, &defender)
  };
  let text = match result {
    BattleResult::Draw => "Draw!",
    BattleResult::Attacker => "Attacker wins!",
    BattleResult::Defender => "Defender wins!"
  };
  println!("{}", text);
}
