#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate rand;
extern crate rodio;
extern crate image;
extern crate tetra_master;

use conrod::{widget, Colorable, Positionable, Widget, Borderable, Sizeable, Labelable};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

use rand::{thread_rng, Rng};

use rodio::{Sink, Source};

use tetra_master::{Board, Space, OwnedCard, Color as CardColor};

use std::fs::File;
use std::io::BufReader;

struct ArrowImages {
  north: conrod::image::Id,
  east: conrod::image::Id,
  south: conrod::image::Id,
  west: conrod::image::Id,
  northeast: conrod::image::Id,
  northwest: conrod::image::Id,
  southeast: conrod::image::Id,
  southwest: conrod::image::Id
}

const WIDTH: u32 = 504;
const HEIGHT: u32 = 744;

fn main() {
  let endpoint = rodio::get_default_endpoint().unwrap();

  let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
  let audio_path = assets.join("audio/Tetra Master loop.ogg");
  let audio = File::open(&audio_path).unwrap();
  let source = rodio::Decoder::new(BufReader::new(audio)).unwrap();
  let mut sink = Sink::new(&endpoint);
  sink.append(source.repeat_infinite());
  sink.set_volume(0.8);

  let display = glium::glutin::WindowBuilder::new()
    .with_dimensions(WIDTH, HEIGHT)
    .with_title("Tetra Master")
    .with_multisampling(4)
    .build_glium()
    .unwrap();

  let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

  widget_ids!(struct Ids {
    cards[],
    grid_spaces[],
    arrows[],
    win_text,
    win_rect,
    volume_slider,
    play_pause_button,
    new_button
  });
  let mut ids = Ids::new(ui.widget_id_generator());

  let font_path = assets.join("fonts/SF/SFDisplay-Regular.ttf");
  ui.fonts.insert_from_file(font_path).unwrap();

  let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

  let mut image_map = conrod::image::Map::new();
  let images = ArrowImages {
    north: image_map.insert(load_image("images/north.png", &display)),
    east: image_map.insert(load_image("images/east.png", &display)),
    south: image_map.insert(load_image("images/south.png", &display)),
    west: image_map.insert(load_image("images/west.png", &display)),
    northeast: image_map.insert(load_image("images/northeast.png", &display)),
    northwest: image_map.insert(load_image("images/northwest.png", &display)),
    southeast: image_map.insert(load_image("images/southeast.png", &display)),
    southwest: image_map.insert(load_image("images/southwest.png", &display))
  };

  let mut board = Board::generate();
  let mut player_hand: Vec<OwnedCard> = (0..5).map(|_| OwnedCard::blue(random::random_card())).collect();
  let mut opponent_hand: Vec<OwnedCard> = (0..5).map(|_| OwnedCard::red(random::random_card())).collect();

  let mut is_player_turn = thread_rng().gen_weighted_bool(2);

  let mut clicked_card: Option<usize> = None;

  let mut last_update = std::time::Instant::now();
  let mut ui_needs_update = true;
  'main: loop {
    let sixteen_ms = std::time::Duration::from_millis(16);
    let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
    if duration_since_last_update < sixteen_ms {
      std::thread::sleep(sixteen_ms - duration_since_last_update);
    }

    let mut events: Vec<_> = display.poll_events().collect();

    if events.is_empty() && !ui_needs_update {
      events.extend(display.wait_events().next());
    }

    ui_needs_update = false;
    last_update = std::time::Instant::now();

    for event in events {
      if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
        ui.handle_event(event);
        ui_needs_update = true;
      }

      match event {
        glium::glutin::Event::Closed => break 'main,
        _ => {}
      }
    }

    if !is_player_turn {
      let mut do_opponent_turn = || {
        is_player_turn = true;
        if opponent_hand.is_empty() {
          return;
        }
        let i = thread_rng().gen_range(0, opponent_hand.len());
        let card = opponent_hand.remove(i);
        let mut empty = Vec::new();
        for r in 0..4 {
          for c in 0..4 {
            if let Space::Empty = *board.space(r + 1, c + 1) {
              empty.push((r, c));
            }
          }
        }
        let (r, c) = match thread_rng().choose(&empty) {
          Some(&(r, c)) => (r, c),
          None => {
            println!("No more empty spaces");
            return;
          }
        };
        board.add_card(r + 1, c + 1, card);
        board.run_battles(r + 1, c + 1);
      };
      do_opponent_turn();
    }

    {
      let ui = &mut ui.set_widgets();

      let volume_events = widget::Slider::new(sink.volume(), 0.0, 1.0)
        .w_h(16.0, 48.0)
        .border_color(conrod::color::GRAY)
        .bottom_right_with_margins_on(ui.window, 36.0, 16.0)
        .set(ids.volume_slider, ui);
      for vol in volume_events {
        sink.set_volume(vol);
      }

      let play_pause_events = widget::Button::new()
        .label(if sink.is_paused() { "Play" } else { "Pause" })
        .w_h(64.0, 32.0)
        .bottom_right_with_margins_on(ui.window, 2.0, 16.0)
        .set(ids.play_pause_button, ui);
      for _ in play_pause_events {
        if sink.is_paused() {
          sink.play();
        } else {
          sink.pause();
        }
      }

      let new_game_events = widget::Button::new()
        .label("New game")
        .w_h(96.0, 32.0)
        .top_left_with_margins_on(ui.window, 2.0, 16.0)
        .set(ids.new_button, ui);
      for _ in new_game_events {
        board = Board::generate();
        player_hand = (0..5).map(|_| OwnedCard::blue(random::random_card())).collect();
        opponent_hand = (0..5).map(|_| OwnedCard::red(random::random_card())).collect();
        is_player_turn = thread_rng().gen_weighted_bool(2);
      }

      ids.cards.resize(player_hand.len(), &mut ui.widget_id_generator());
      for (i, card) in player_hand.iter().enumerate() {
        let (x, y) = match i {
          0 => (
            -(ui.window_dim()[0] / 2.0) + 250.0 + 2.0,
            (ui.window_dim()[1] / 2.0) - 74.0 - 2.0
          ),
          1 => (
            -(ui.window_dim()[0] / 2.0) + 350.0 + 2.0,
            (ui.window_dim()[1] / 2.0) - 74.0 - 2.0
          ),
          2 => (
            -(ui.window_dim()[0] / 2.0) + 450.0 + 2.0,
            (ui.window_dim()[1] / 2.0) - 74.0 - 2.0
          ),
          3 => (
            -(ui.window_dim()[0] / 2.0) + 450.0 + 2.0,
            (ui.window_dim()[1] / 2.0) - 222.0 - 2.0
          ),
          4 => (
            -(ui.window_dim()[0] / 2.0) + 450.0 + 2.0,
            (ui.window_dim()[1] / 2.0) - 370.0 - 2.0
          ),
          _ => panic!("Hand too large")
        };
        let card_id = ids.cards.get(i).unwrap().clone();
        let label = &card.to_string();
        let (card_id, arrows, mut card_button) = owned_card_to_game_card(&images, card_id, widget::Button::new(), card);
        card_button = card_button
          .label(label)
          .x_y(x, y);

        for _click in card_button.clone().set(card_id, ui) {
          clicked_card = Some(i);
        }

        let amount_of_arrows = ids.arrows.len();
        ids.arrows.resize(amount_of_arrows + arrows.len(), &mut ui.widget_id_generator());
        for (i, arrow) in arrows.into_iter().enumerate() {
          arrow.set(ids.arrows.get(amount_of_arrows + i).unwrap().clone(), ui);
        }
      }

      ids.grid_spaces.resize(16, &mut ui.widget_id_generator());
      let mut id_count = 0;
      for row in 0..4 {
        for col in 0..4 {
          let x: f64 = -(ui.window_dim()[0] / 2.0) + (col as f64 * 100.0) + 52.0;
          let y: f64 = (ui.window_dim()[1] / 2.0) - (row as f64 * 148.0) - 224.0;
          let button = widget::Button::new();
          let mut button_id = ids.grid_spaces.get(id_count).unwrap().clone();
          let (label, arrows, mut button) = match *board.space(row + 1, col + 1) {
            Space::Block => (String::new(), Vec::new(), button.color(conrod::color::DARK_GRAY)),
            Space::Card(ref c) => {
              let (id, arrows, button) = owned_card_to_game_card(&images, button_id, button, c);
              button_id = id;
              (c.to_string(), arrows, button)
            },
            Space::Empty => (String::new(), Vec::new(), button.color(conrod::color::BLACK))
          };
          let label = &label;
          button = button
            .label(label)
            .w_h(100.0, 148.0)
            .border_color(conrod::color::WHITE)
            .x_y(x, y);
          for _click in button.set(button_id, ui) {
            if let Some(i) = clicked_card {
              if let Space::Empty = *board.space(row + 1, col + 1) {
                is_player_turn = false;
                let card = player_hand.remove(i);
                board.add_card(row + 1, col + 1, card);
                board.run_battles(row + 1, col + 1);
                clicked_card = None;
              }
            }
          }
          let amount_of_arrows = ids.arrows.len();
          ids.arrows.resize(amount_of_arrows + arrows.len(), &mut ui.widget_id_generator());
          for (i, arrow) in arrows.into_iter().enumerate() {
            arrow.set(ids.arrows.get(amount_of_arrows + i).unwrap().clone(), ui);
          }
          id_count += 1;
        }
      }

      if opponent_hand.is_empty() && player_hand.is_empty() {
        let cards: Vec<&OwnedCard> = board.spaces
          .iter()
          .flat_map(|x| x.iter().collect::<Vec<&Space>>())
          .filter(|s| s.is_card())
          .map(|x| match *x {
            Space::Card(ref c) => c,
            _ => unreachable!()
          })
          .collect();
        let blue = cards.iter().filter(|x| x.color.get() == CardColor::Blue).count();
        let red = cards.iter().filter(|x| x.color.get() == CardColor::Red).count();
        let text = if blue == red {
          "Draw"
        } else if blue > red {
          "Blue wins"
        } else {
          "Red wins"
        };
        let text_widget = widget::Text::new(text).font_size(48);
        let x_dim = match text_widget.default_x_dimension(ui) {
          conrod::position::Dimension::Absolute(x) => x,
          _ => panic!()
        };
        let y_dim = match text_widget.default_y_dimension(ui) {
          conrod::position::Dimension::Absolute(x) => x,
          _ => panic!()
        };
        widget::Rectangle::fill_with([x_dim + 8.0, y_dim + 8.0], conrod::color::BLACK)
          .middle_of(ui.window)
          .set(ids.win_rect, ui);
        text_widget
          .color(conrod::color::WHITE)
          .middle_of(ids.win_rect)
          .set(ids.win_text, ui);
      }
    }

    if let Some(primitives) = ui.draw_if_changed() {
      renderer.fill(&display, primitives, &image_map);
      let mut target = display.draw();
      target.clear_color(0.0, 0.0, 0.0, 1.0);
      renderer.draw(&display, &mut target, &image_map).unwrap();
      target.finish().unwrap();
    }
  }
}

fn load_image(url: &str, display: &glium::Display) -> glium::texture::Texture2d {
  let assets = find_folder::Search::ParentsThenKids(3, 5).for_folder("assets").unwrap();
  let path = assets.join(url);
  let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
  let image_dimensions = rgba_image.dimensions();
  let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(rgba_image.into_raw(), image_dimensions);
  let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
  texture
}

fn owned_card_to_game_card<'a>(images: &ArrowImages, id: widget::Id, base: widget::Button<'a, widget::button::Flat>, card: &OwnedCard) -> (widget::Id, Vec<widget::Image>, widget::Button<'a, widget::button::Flat>) {
  let mut arrows = Vec::new();
  if card.arrows.north() {
    arrows.push(widget::Image::new(images.north)
      .w_h(16.0, 8.0)
      .mid_top_with_margin_on(id, 4.0));
  }
  if card.arrows.northeast() {
    arrows.push(widget::Image::new(images.northeast)
      .w_h(16.0, 16.0)
      .top_right_with_margins_on(id, 4.0, 4.0));
  }
  if card.arrows.east() {
    arrows.push(widget::Image::new(images.east)
      .w_h(8.0, 16.0)
      .mid_right_with_margin_on(id, 4.0));
  }
  if card.arrows.southeast() {
    arrows.push(widget::Image::new(images.southeast)
      .w_h(16.0, 16.0)
      .bottom_right_with_margins_on(id, 4.0, 4.0));
  }
  if card.arrows.south() {
    arrows.push(widget::Image::new(images.south)
      .w_h(16.0, 8.0)
      .mid_bottom_with_margin_on(id, 4.0));
  }
  if card.arrows.southwest() {
    arrows.push(widget::Image::new(images.southwest)
      .w_h(16.0, 16.0)
      .bottom_left_with_margins_on(id, 4.0, 4.0));
  }
  if card.arrows.west() {
    arrows.push(widget::Image::new(images.west)
      .w_h(8.0, 16.0)
      .mid_left_with_margin_on(id, 4.0));
  }
  if card.arrows.northwest() {
    arrows.push(widget::Image::new(images.northwest)
      .w_h(16.0, 16.0)
      .top_left_with_margins_on(id, 4.0, 4.0));
  }
  let button = base
    .label_font_size(16)
    .label_color(conrod::color::BLACK)
    .label_x(conrod::position::Relative::Scalar(0.0))
    .label_y(conrod::position::Relative::Scalar(-48.0))
    .center_justify_label()
    .w_h(96.0, 144.0)
    .color(if let tetra_master::Color::Blue = card.color.get() { conrod::color::LIGHT_BLUE } else { conrod::color::ORANGE });
  (id, arrows, button)
}

mod random {
  use tetra_master::*;
  use rand::{thread_rng, Rng};

  /// Get a random card for a player of the given level. Level is between [1, 100] and increases the
  /// chance of generating better cards the higher it is.
  pub fn random_card() -> Card {
    let power = weighted_level();
    let class = match thread_rng().gen_range(0, 100) {
      0...39 => Class::Physical,
      40...80 => Class::Magical,
      81...95 => Class::Flexible,
      96...100 => Class::Assault,
      _ => panic!("Unexpected random number")
    };
    let phys_def = weighted_level();
    let mag_def = weighted_level();
    Card::with_arrows(power, class, phys_def, mag_def, random_arrows())
  }

  fn random_arrows() -> Arrows {
    let mut flags = 0;
    for i in 0..8 {
      let chance = if flags == 0 { 2 } else { 4 };
      if thread_rng().gen_weighted_bool(chance) {
        flags |= 1 << i;
      }
    }
    Arrows::from_flags(flags)
  }

  const WEIGHTS: &'static [u8] = &[
    15,
    15,
    15,
    8,
    8,
    8,
    5,
    5,
    5,
    3,
    3,
    3,
    2,
    2,
    2,
    1
  ];

  fn weighted_level() -> u8 {
    let weight_sum = WEIGHTS.iter().sum();
    let mut random_weight = thread_rng().gen_range(0, weight_sum);
    for (i, item) in WEIGHTS.iter().enumerate() {
      if random_weight < *item {
        return i as u8;
      }
      random_weight -= *item;
    }
    unreachable!();
  }
}
