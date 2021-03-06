/**
 * Chess GUI template.
 * Author: Eskil Queseth <eskilq@kth.se>, Viola Söderlund <violaso@kth.se>
 * Last updated: 2020-09-20
 */

use ggez::event;
use ggez::timer;
use ggez::graphics::{self, DrawParam, Color, DrawMode};
use ggez::{Context, GameResult};
use std::path;
use eliasfl_chess::{Game, GameState, Color as Colour, Piece as PieceType, Position};
use ggez::event::{MouseButton};
use std::collections::HashMap;
use std::collections::HashSet;
use rand::Rng;
use rand::rngs;


static mut seed: u64 = 0;

/// A chess board is 8x8 tiles.
const GRID_SIZE: (i16, i16) = (8, 8);
/// Sutible size of each tile.
const GRID_CELL_SIZE: (i16, i16) = (45, 45);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32 * 3.0,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32 * 1.5,
);

// GUI Color representations
const BLACK: Color = Color::new(228.0/255.0, 196.0/255.0, 108.0/255.0, 1.0);
const WHITE: Color = Color::new(188.0/255.0, 140.0/255.0, 76.0/255.0, 1.0);
const BLACK_RED: Color = Color::new(255.0/255.0, 96.0/255.0, 78.0/255.0, 1.0);
const WHITE_RED: Color = Color::new(215.0/255.0, 69.0/255.0, 60.0/255.0, 1.0);
const SEMI_OPAQUE_GREY: Color = Color::new(0.5, 0.5, 0.5, 0.2);

// Enumerable over possible modifications for a player. 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mods {
    CrazyHouse(PieceType),
    Atomic(PieceType),
    Sniper(PieceType),
    KingOfTheHill,
    Extinction(PieceType),
    // Hidden(PieceType), Only works with networking... will need to be rethought somewhat
    TripleCheck(PieceType)
}

impl Mods {
    fn to_string(&self) -> &str {
        match self {
            Mods::CrazyHouse(_) => "CrazyHouse",
            Mods::Atomic(_) => "Atomic",
            Mods::Sniper(_) => "Sniper",
            Mods::KingOfTheHill => "King of the Hill",
            Mods::Extinction(_) => "Extinction",
            Mods::TripleCheck(_) => "Triple Check",
        }
    }

    fn get_piece(&self, col: Colour) -> PieceType {
        let not_col;
        if col == Colour::White { not_col = Colour::Black } else { not_col = Colour::White }
        match self {
            Mods::CrazyHouse(piece) => piece.type_as_colour(not_col),
            Mods::Atomic(piece) => piece.type_as_colour(col),
            Mods::Sniper(piece) => piece.type_as_colour(col),
            Mods::KingOfTheHill => PieceType::King(col),
            Mods::Extinction(piece) => piece.type_as_colour(not_col),
            Mods::TripleCheck(piece) => piece.type_as_colour(col),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScreenState {
    GameScreen,
    ScoreScreen,
    ModScreen,
}

/// GUI logic and event implementation structure. 
struct AppState {
    sprites: HashMap<PieceType, graphics::Image>,
    board: Game,
    selected_pos: (isize, isize),
    highlighted_pos: Vec<(isize, isize)>,
    taken_black_pieces: Vec<PieceType>,
    taken_white_pieces: Vec<PieceType>,
    white_mods: HashSet<Mods>,
    black_mods: HashSet<Mods>,
    triple_check_counter: (u8, u8),
    wins: (u8, u8),
    screen: ScreenState,
    cur_winner: Option<Colour>,
    random_mods: Vec<Mods>,
}

impl AppState {
    /// Initialise new application, i.e. initialise new game and load resources.
    fn new(ctx: &mut Context) -> GameResult<AppState> {
        let sprites = AppState::load_sprites();
        let mut board = Game::new();
        board.set_promotion("queen".to_string());

        let state = AppState {
            sprites: sprites
                .iter()
                .map(|_sprite| {
                    (_sprite.0, graphics::Image::new(ctx, _sprite.1.clone()).unwrap())
                })
                .collect::<HashMap<PieceType, graphics::Image>>(),
            board: board,
            selected_pos: (0, 0),
            highlighted_pos: Vec::new(),
            taken_black_pieces: Vec::new(),
            taken_white_pieces: Vec::new(),
            white_mods: HashSet::new(),
            black_mods: HashSet::new(),
            triple_check_counter: (0, 0),
            wins: (0, 0),
            screen: ScreenState::GameScreen,
            cur_winner: None,
            random_mods: Vec::new(),
        };

        Ok(state)
    }

    /// Loads chess piese images into vector.
    fn load_sprites() -> Vec<(PieceType, String)> {
        let mut sprites = Vec::new();
        sprites.push(((PieceType::King(Colour::Black)), "/black_king.png".to_string()));
        sprites.push(((PieceType::Queen(Colour::Black)), "/black_queen.png".to_string()));
        sprites.push(((PieceType::Rook(Colour::Black)), "/black_rook.png".to_string()));
        sprites.push(((PieceType::Pawn(Colour::Black)), "/black_pawn.png".to_string()));
        sprites.push(((PieceType::Bishop(Colour::Black)), "/black_bishop.png".to_string()));
        sprites.push(((PieceType::Knight(Colour::Black)), "/black_knight.png".to_string()));
        sprites.push(((PieceType::King(Colour::White)), "/white_king.png".to_string()));
        sprites.push(((PieceType::Queen(Colour::White)), "/white_queen.png".to_string()));
        sprites.push(((PieceType::Rook(Colour::White)), "/white_rook.png".to_string()));
        sprites.push(((PieceType::Pawn(Colour::White)), "/white_pawn.png".to_string()));
        sprites.push(((PieceType::Bishop(Colour::White)), "/white_bishop.png".to_string()));
        sprites.push(((PieceType::Knight(Colour::White)), "/white_knight.png".to_string()));
        sprites
    }

    fn end_game(&mut self, winner: Option<Colour>) {
        self.screen = ScreenState::ScoreScreen;
        self.cur_winner = winner;
    }
}

/// Implement each stage of the application event loop. 
impl event::EventHandler for AppState {

    /// For updating game logic, which front-end doesn't handle.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.board.get_game_state() == GameState::CheckMate && self.screen == ScreenState::GameScreen {
            match self.board.active_color {
                Colour::White => {
                    self.end_game(Some(Colour::Black));
                },
                Colour::Black => {
                    self.end_game(Some(Colour::White));
                }
            }
        }

        unsafe { seed += 1; }

        Ok(())
    }

    /// Draw interface, i.e. draw game board
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear interface with gray background colour
        graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());

        if self.screen == ScreenState::GameScreen || self.screen == ScreenState::ScoreScreen {
            // create text representation
            let state_text = graphics::Text::new(
                    graphics::TextFragment::from(format!("Game is {:?}.", self.board.get_game_state())
                )
                .scale(graphics::Scale { x: 20.0, y: 20.0 }));
            let turn_text = graphics::Text::new(
                    graphics::TextFragment::from(format!("Current turn is {:?}.", self.board.active_color)
                )
                .scale(graphics::Scale { x: 18.5, y: 20.0 })); //dont ask
            let promotion_text = graphics::Text::new(
                    graphics::TextFragment::from(format!("Current promotion:")
                )
                .scale(graphics::Scale { x: 20.0, y: 20.0 }));
            let black_mod_title = graphics::Text::new(
                graphics::TextFragment::from(format!("Black mods:")
            )
                .scale(graphics::Scale { x: 20.0, y: 20.0 }));
            let white_mod_title = graphics::Text::new(
                graphics::TextFragment::from(format!("White mods:")
            )
                .scale(graphics::Scale { x: 20.0, y: 20.0 }));

            // get size of text
            let text_dimensions = state_text.dimensions(ctx);
            // create background rectangle with white coulouring
            let background_box = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(),
                graphics::Rect::new(0f32,
                                    0f32,
                                    SCREEN_SIZE.0, SCREEN_SIZE.1),
                                    [1.0, 1.0, 1.0, 1.0].into()
            )?;

            // draw background
            graphics::draw(ctx, &background_box, DrawParam::default());

            // draw tiles
            for i in 0..64 {
                let rectangle = graphics::Mesh::new_rectangle(ctx, 
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new_i32(
                        (i % 8 * GRID_CELL_SIZE.0 as i32) + (SCREEN_SIZE.0 as i32 / 4),
                        i / 8 * GRID_CELL_SIZE.1 as i32,
                        GRID_CELL_SIZE.0 as i32,
                        GRID_CELL_SIZE.1 as i32,
                    ), if int_to_pos_tuple(i as isize) == self.selected_pos || self.highlighted_pos.contains(&int_to_pos_tuple(i as isize)) { if (int_to_pos_tuple(i as isize).0 % 2 == 0) ^ (int_to_pos_tuple(i as isize).1 % 2 == 0) { BLACK_RED } else { WHITE_RED } }
                    else { match i % 2 {
                        0 => match i / 8 {
                            _row if _row % 2 == 0 => WHITE,
                            _ => BLACK
                        },
                        _ => match i / 8 {
                            _row if _row % 2 == 0 => BLACK,
                            _ => WHITE
                        }
                    }})?;
                graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
            }

            // draw selected taken piece
            if self.selected_pos.1 == 9 || self.selected_pos.1 == 10 {
                let rectangle = graphics::Mesh::new_rectangle(ctx, 
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new_i32(
                        (self.selected_pos.0 * GRID_CELL_SIZE.0 as isize) as i32 + ((SCREEN_SIZE.0 * 0.25) - GRID_CELL_SIZE.0 as f32) as i32,
                        self.selected_pos.1 as i32 * GRID_CELL_SIZE.1 as i32,
                        GRID_CELL_SIZE.0 as i32,
                        GRID_CELL_SIZE.1 as i32,
                    ), BLACK_RED)?;
                graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
            }

            // draw pieces
            for (pos, val) in self.board.board.iter() {
                graphics::draw(ctx, &self.sprites[val], (ggez::mint::Point2 { x: ((pos.file - 1) as f32 * GRID_CELL_SIZE.0 as f32) + SCREEN_SIZE.0 * 0.25 as f32, y: (8 - pos.rank) as f32 * GRID_CELL_SIZE.1 as f32 }, ));
            }

            // draw taken pieces
            for x in 0..self.taken_black_pieces.len() {
                graphics::draw(ctx, &self.sprites[&self.taken_black_pieces[x]], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.25 - GRID_CELL_SIZE.0 as f32 + (GRID_CELL_SIZE.0 as usize * x) as f32, y: 9f32 * GRID_CELL_SIZE.1 as f32 }, ));
            }
            for x in 0..self.taken_white_pieces.len() {
                graphics::draw(ctx, &self.sprites[&self.taken_white_pieces[x]], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.25 - GRID_CELL_SIZE.0 as f32 + (GRID_CELL_SIZE.0 as usize * x) as f32, y: 10f32 * GRID_CELL_SIZE.1 as f32 }, ));
            }

            // draw promotion selectors
            let current_color = self.board.active_color;
            let turn_idx = if current_color == Colour::White { 0 } else { 1 };
            let promo_y = match self.board.promotion[turn_idx] {
                PieceType::Queen(current_color) => (SCREEN_SIZE.0 * 0.75) as i32,
                PieceType::Rook(current_color) => (SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32) as i32,
                PieceType::Bishop(current_color) => (SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32 * 2f32) as i32,
                PieceType::Knight(current_color) => (SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32 * 3f32) as i32,
                _ => panic!(),
            };
            let rectangle = graphics::Mesh::new_rectangle(ctx, 
                graphics::DrawMode::fill(), 
                graphics::Rect::new_i32(
                    promo_y,
                    (GRID_CELL_SIZE.1 * 3) as i32,
                    GRID_CELL_SIZE.0 as i32,
                    GRID_CELL_SIZE.1 as i32,
                ), BLACK_RED)?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));

            graphics::draw(ctx, &self.sprites[&PieceType::Queen(self.board.active_color)], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.75, y: 3f32 * GRID_CELL_SIZE.1 as f32 }, ));
            graphics::draw(ctx, &self.sprites[&PieceType::Rook(self.board.active_color)], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32, y: 3f32 * GRID_CELL_SIZE.1 as f32 }, ));
            graphics::draw(ctx, &self.sprites[&PieceType::Bishop(self.board.active_color)], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32 * 2f32, y: 3f32 * GRID_CELL_SIZE.1 as f32 }, ));
            graphics::draw(ctx, &self.sprites[&PieceType::Knight(self.board.active_color)], (ggez::mint::Point2 { x: SCREEN_SIZE.0 * 0.75 + GRID_CELL_SIZE.0 as f32 * 3f32, y: 3f32 * GRID_CELL_SIZE.1 as f32 }, ));
            
            // draw text with dark gray colouring and center position
            graphics::draw(ctx, &state_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: SCREEN_SIZE.0 * 0.75,
                    y: 0f32,
                }));
            graphics::draw(ctx, &turn_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: SCREEN_SIZE.0 * 0.75,
                    y: GRID_CELL_SIZE.1 as f32 * 1f32,
                }));
            graphics::draw(ctx, &promotion_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: SCREEN_SIZE.0 * 0.75,
                    y: GRID_CELL_SIZE.1 as f32 * 2f32,
                }));
            graphics::draw(ctx, &white_mod_title, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: 0f32,
                    y: 0f32,
                }));
            {
                let mut x = 0.0;
            for modi in self.white_mods.iter() {
                let text = graphics::Text::new(
                    graphics::TextFragment::from(format!("{:?} {:?}", modi.to_string(), modi.get_piece(Colour::White))
                )
                    .scale(graphics::Scale { x: 20.0, y: 20.0 }));
                    x += 1.0;
                graphics::draw(ctx, &text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: 0f32,
                    y: 25f32 * x,
                }));
                } 
            }
            graphics::draw(ctx, &black_mod_title, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: 0f32,
                    y: SCREEN_SIZE.1 * 0.5,
                }));
                {
                    let mut x = 0.0;
                for modi in self.black_mods.iter() {
                    let text = graphics::Text::new(
                        graphics::TextFragment::from(format!("{:?} {:?}", modi.to_string(), modi.get_piece(Colour::Black))
                    )
                        .scale(graphics::Scale { x: 20.0, y: 20.0 }));
                        x += 1.0;
                    graphics::draw(ctx, &text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                    .dest(ggez::mint::Point2 {
                        x: 0f32,
                        y: SCREEN_SIZE.1 * 0.5 + 25f32 * x,
                    }));
                    } 
                }
            
            
            // draw score if relevant
            if (self.screen == ScreenState::ScoreScreen) {
            let cover_rectangle = graphics::Mesh::new_rectangle(ctx, 
                graphics::DrawMode::fill(), 
                graphics::Rect::new_i32(
                    0,
                    0,
                    SCREEN_SIZE.0 as i32,
                    SCREEN_SIZE.1 as i32,
                ), SEMI_OPAQUE_GREY)?;
            graphics::draw(ctx, &cover_rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
            
            let mut score_text;
            if self.cur_winner == None {
                score_text = graphics::Text::new(
                        graphics::TextFragment::from(format!("No one wins!")
                    )
                    .scale(graphics::Scale { x: 60.0, y: 60.0 }));
            } else {
                score_text = graphics::Text::new(
                        graphics::TextFragment::from(format!("{:?} wins!", self.cur_winner.unwrap())
                    )
                    .scale(graphics::Scale { x: 60.0, y: 60.0 }));
            }
            let text_dim = score_text.dimensions(ctx); 
            graphics::draw(ctx, &score_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: (SCREEN_SIZE.0 - text_dim.0 as f32) * 0.5,
                    y: (SCREEN_SIZE.1 - text_dim.1 as f32) * 0.5,
                }));
            }
        }
        
        if self.screen == ScreenState::ModScreen {
            // draw background
            let background = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(),
            graphics::Rect::new(0f32,
                            0f32,
                            SCREEN_SIZE.0,
                            SCREEN_SIZE.1),
                            [0.5, 0.5, 0.5, 1.0].into()
                        )?;

            // draw boxes for the choices.
            let inner_box = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(),
                graphics::Rect::new(GRID_CELL_SIZE.0 as f32 * 0.5,
                            GRID_CELL_SIZE.1 as f32 * 0.5,
                            SCREEN_SIZE.0 - GRID_CELL_SIZE.0 as f32,
                            (SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32),
                            [1.0, 1.0, 1.0, 1.0].into()
            )?;

            let bounding_box = graphics::Mesh::new_rectangle(ctx, DrawMode::stroke(3f32),
                graphics::Rect::new(GRID_CELL_SIZE.0 as f32 * 0.5,
                                GRID_CELL_SIZE.1 as f32 * 0.5,
                                SCREEN_SIZE.0 - GRID_CELL_SIZE.0 as f32,
                                (SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32),
                                [0.0, 0.0, 0.0, 1.0].into()
            )?;
            graphics::draw(ctx, &inner_box, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
            graphics::draw(ctx, &inner_box, (ggez::mint::Point2 { x: 0.0, y: ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) + GRID_CELL_SIZE.1 as f32 * 0.5 }, ));
            graphics::draw(ctx, &inner_box, (ggez::mint::Point2 { x: 0.0, y: ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 1.5) + GRID_CELL_SIZE.1 as f32 }, ));
            
            graphics::draw(ctx, &bounding_box, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
            graphics::draw(ctx, &bounding_box, (ggez::mint::Point2 { x: 0.0, y: ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) + GRID_CELL_SIZE.1 as f32 * 0.5 }, ));
            graphics::draw(ctx, &bounding_box, (ggez::mint::Point2 { x: 0.0, y: ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 1.5) + GRID_CELL_SIZE.1 as f32 }, ));

            // Draw different text for each choice
            let mod_1_text = graphics::Text::new(
                graphics::TextFragment::from(format!("{:?} {:?}", self.random_mods[0].to_string(), self.random_mods[0].get_piece(self.cur_winner.unwrap()))
            )
            .scale(graphics::Scale { x: 20.0, y: 20.0 }));
            let mod_2_text = graphics::Text::new(
                graphics::TextFragment::from(format!("{:?} {:?}", self.random_mods[1].to_string(), self.random_mods[1].get_piece(self.cur_winner.unwrap()))
            )
            .scale(graphics::Scale { x: 20.0, y: 20.0 }));
            let mod_3_text = graphics::Text::new(
                graphics::TextFragment::from(format!("{:?} {:?}", self.random_mods[2].to_string(), self.random_mods[2].get_piece(self.cur_winner.unwrap()))
            )
            .scale(graphics::Scale { x: 20.0, y: 20.0 }));

            graphics::draw(ctx, &mod_1_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: GRID_CELL_SIZE.0 as f32 * 2f32,
                    y: GRID_CELL_SIZE.1 as f32,
                }));
            graphics::draw(ctx, &self.sprites[&self.random_mods[0].get_piece(self.cur_winner.unwrap())], (ggez::mint::Point2 { x: GRID_CELL_SIZE.0 as f32, y: GRID_CELL_SIZE.1 as f32 }, ));
            graphics::draw(ctx, &mod_2_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: GRID_CELL_SIZE.0 as f32 * 2f32,
                    y: GRID_CELL_SIZE.1 as f32 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) + GRID_CELL_SIZE.1 as f32 * 0.5,
                }));
                graphics::draw(ctx, &self.sprites[&self.random_mods[1].get_piece(self.cur_winner.unwrap())], (ggez::mint::Point2 { x: GRID_CELL_SIZE.0 as f32, y: GRID_CELL_SIZE.1 as f32 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) + GRID_CELL_SIZE.1 as f32 * 0.5 as f32 }, ));
            graphics::draw(ctx, &mod_3_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: GRID_CELL_SIZE.0 as f32 * 2f32,
                    y: GRID_CELL_SIZE.1 as f32 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 1.5) + GRID_CELL_SIZE.1 as f32,
                }));
                graphics::draw(ctx, &self.sprites[&self.random_mods[2].get_piece(self.cur_winner.unwrap())], (ggez::mint::Point2 { x: GRID_CELL_SIZE.0 as f32, y: GRID_CELL_SIZE.1 as f32 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 1.5) + GRID_CELL_SIZE.1 as f32 as f32 }, ));
        }
        // render updated graphics
        graphics::present(ctx)?;

        Ok(())
    }

    /// Update game on mouse click
    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left && self.screen == ScreenState::ModScreen {
            if x > GRID_CELL_SIZE.0 as f32 * 0.5 && x < SCREEN_SIZE.0 - (GRID_CELL_SIZE.0 as f32 * 0.5) {
                match y {
                    y if y > GRID_CELL_SIZE.1 as f32 * 0.5 && y < GRID_CELL_SIZE.1 as f32 * 0.5 + (SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32 => {
                        if self.cur_winner.unwrap() == Colour::White {
                            self.black_mods.insert(self.random_mods[0]);
                        } else {
                            self.white_mods.insert(self.random_mods[0]);
                        }
                        self.screen = ScreenState::GameScreen;
                        self.board = Game::new();
                        self.random_mods = Vec::new();
                        return;
                    }
                    y if y > GRID_CELL_SIZE.1 as f32 * 0.5 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) + GRID_CELL_SIZE.1 as f32 * 0.5 && y < (GRID_CELL_SIZE.1 as f32 * 0.5 + (SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) * 2f32 => {
                        if self.cur_winner.unwrap() == Colour::White {
                            self.black_mods.insert(self.random_mods[1]);
                        } else {
                            self.white_mods.insert(self.random_mods[1]);
                        }
                        self.screen = ScreenState::GameScreen;
                        self.board = Game::new();
                        self.random_mods = Vec::new();
                        return;
                    }
                    y if y > GRID_CELL_SIZE.1 as f32 * 0.5 + ((SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 1.5) + GRID_CELL_SIZE.1 as f32 && y < (GRID_CELL_SIZE.1 as f32 * 0.5 + (SCREEN_SIZE.1 - (GRID_CELL_SIZE.1 as f32 * 2f32)) / 3f32) * 3f32 => {
                        if self.cur_winner.unwrap() == Colour::White {
                            self.black_mods.insert(self.random_mods[2]);
                        } else {
                            self.white_mods.insert(self.random_mods[2]);
                        }
                        self.screen = ScreenState::GameScreen;
                        self.board = Game::new();
                        self.random_mods = Vec::new();
                        return;
                    }
                    _ => {},
                }
            }
        } else if button == MouseButton::Left && self.screen == ScreenState::ScoreScreen {
            if self.cur_winner == None {
                self.screen = ScreenState::GameScreen;
                self.board = Game::new();
                return;
            }
            let cur_loser;
            if self.cur_winner.unwrap() == Colour::White { cur_loser = Colour::Black } else { cur_loser = Colour::White }
            unsafe {
            let mut rng: rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
            seed += 1;
            self.screen = ScreenState::ModScreen;
            let (mut mod_1, mut mod_2, mut mod_3) = (generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen()), generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen()), generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen()));
            if cur_loser == Colour::White {
                while (self.white_mods.contains(&mod_1)) {
                    mod_1 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
                while (self.white_mods.contains(&mod_2)) {
                    mod_2 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
                while (self.white_mods.contains(&mod_3)) {
                    mod_3 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
            } else {
                while (self.black_mods.contains(&mod_1)) {
                    mod_1 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
                while (self.black_mods.contains(&mod_2)) {
                    mod_2 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
                while (self.black_mods.contains(&mod_3)) {
                    mod_3 = generate_mod(self.cur_winner.unwrap(), rng.gen(), rng.gen());
                }
            }
            self.random_mods.push(mod_1);
            self.random_mods.push(mod_2);
            self.random_mods.push(mod_3);
        }
            println!("{:?}", self.screen);
        } else if button == MouseButton::Left && self.screen == ScreenState::GameScreen {
            if x <= SCREEN_SIZE.0 * 0.75 && x >= SCREEN_SIZE.0 * 0.25 && y < SCREEN_SIZE.1 * 2f32 / 3f32 {
                let pos_x = x - (SCREEN_SIZE.0 * 0.25f32);
                let pos_x = (pos_x / GRID_CELL_SIZE.0 as f32).ceil();
                let pos_y = 9f32 - (y / GRID_CELL_SIZE.1 as f32).ceil();

                if self.highlighted_pos.contains(&(pos_x as isize, pos_y as isize)) {
                    let mut taking_move = false;
                    if self.board.board.contains_key(&Position { file: pos_x as u8, rank: pos_y as u8 }) && self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}].colour() == self.board.active_color {
                        taking_move = true;
                        match self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }] {
                            PieceType::Queen(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::Queen(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::Queen(Colour::White))
                            },
                            PieceType::King(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::King(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::King(Colour::White))
                            },
                            PieceType::Pawn(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::Pawn(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::Pawn(Colour::White))
                            },
                            PieceType::Bishop(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::Bishop(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::Bishop(Colour::White))
                            },
                            PieceType::Knight(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::Knight(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::Knight(Colour::White))
                            },
                            PieceType::Rook(colour) => match colour {
                                Colour::Black => self.taken_black_pieces.push(PieceType::Rook(Colour::Black)),
                                _ => self.taken_white_pieces.push(PieceType::Rook(Colour::White))
                            },
                        }
                    }
                    if self.board.board.contains_key(&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}) && self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}].colour() == self.board.active_color {
                        let mut sniper = false;
                        if self.board.board.contains_key(&Position { file: pos_x as u8, rank: pos_y as u8 }) {
                            match self.board.active_color {
                                Colour::Black => {
                                    if self.black_mods.contains(&Mods::Sniper(self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}])) {
                                        sniper = true;
                                    }
                                    if self.black_mods.contains(&Mods::Atomic(self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}])) {
                                        for x in 0..=2 {
                                            for y in 0..=2 {
                                                if x == 1 && y == 1 { continue; }
                                                if self.board.board.contains_key(&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }) {
                                                    match self.board.board[&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }] {
                                                        PieceType::Pawn(_colour) => (),
                                                        _ => { 
                                                            self.taken_white_pieces.push(self.board.board[&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }]);
                                                            self.board.board.remove(&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }); 
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                        if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) && !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(None);
                                        } else if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(Some(Colour::Black));
                                        } else if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(Some(Colour::White));
                                        }
                                    }
                                    if self.black_mods.contains(&Mods::Extinction(self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }])) {
                                        let mut theoretical_board = self.board.board.clone();
                                        theoretical_board.remove(&Position { file: pos_x as u8, rank: pos_y as u8 });
                                        if theoretical_board.values().any(|x| x == &self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }]) {
                                            self.board.make_move(Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8 }.to_string(), Position { file: pos_x as u8, rank: pos_y as u8 }.to_string());
                                            self.end_game(Some(Colour::Black));
                                        }
                                    }
                                }
                                Colour::White => {
                                    if self.white_mods.contains(&Mods::Sniper(self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}])) {
                                        sniper = true;
                                    }
                                    if self.white_mods.contains(&Mods::Atomic(self.board.board[&Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8}])) {
                                        for x in 0..=2 {
                                            for y in 0..=2 {
                                                if self.board.board.contains_key(&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }) {
                                                    match self.board.board[&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }] {
                                                        PieceType::Pawn(_colour) => (),
                                                        _ => { 
                                                            self.taken_black_pieces.push(self.board.board[&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }]);
                                                            self.board.board.remove(&Position { file: (pos_x + x as f32 - 1f32) as u8, rank: (pos_y + y as f32 - 1f32) as u8 }); 
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                        if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) && !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(None);
                                        } else if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(Some(Colour::Black));
                                        } else if !self.board.board.values().any(|x| x == &PieceType::King(Colour::White)) {
                                            self.end_game(Some(Colour::White));
                                        }
                                    }
                                    if self.white_mods.contains(&Mods::Extinction(self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }])) {
                                        let mut theoretical_board = self.board.board.clone();
                                        theoretical_board.remove(&Position { file: pos_x as u8, rank: pos_y as u8 });
                                        if theoretical_board.values().any(|x| x == &self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }]) {
                                            self.board.make_move(Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8 }.to_string(), Position { file: pos_x as u8, rank: pos_y as u8 }.to_string());
                                            self.end_game(Some(Colour::White));
                                        }
                                    }
                                }
                            }
                        }
                        let successful = self.board.make_move(Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8 }.to_string(), Position { file: pos_x as u8, rank: pos_y as u8 }.to_string()).is_ok();
                        if sniper {
                            self.board.board.insert(Position { file: self.selected_pos.0 as u8, rank: self.selected_pos.1 as u8 }, self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }]);
                            self.board.board.remove(&Position { file: pos_x as u8, rank: pos_y as u8 });
                        }
                        if successful && self.board.board.contains_key(&Position { file: pos_x as u8, rank: pos_y as u8 }){
                            match self.board.active_color {
                                Colour::White => { 
                                    let p = self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }];
                                    if self.black_mods.contains(&Mods::TripleCheck(p)) && self.board.get_game_state() == GameState::Check {
                                        if self.board.board.iter_mut().any(|(k, v)| v == &mut PieceType::King(Colour::White) && p.valid_destinations(&Position { file: pos_x as u8, rank: pos_y as u8 }).contains(&k)) {
                                            self.triple_check_counter = (self.triple_check_counter.0, self.triple_check_counter.1 + 1);
                                            if self.triple_check_counter.1 >= 3 {
                                                self.end_game(Some(Colour::Black));
                                            }
                                        }
                                    }
                                    if (self.white_mods.contains(&Mods::Atomic(p))) && !sniper && taking_move {
                                        self.taken_black_pieces.push(self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }]);
                                        self.board.board.remove(&Position { file: pos_x as u8, rank: pos_y as u8 });
                                    } 
                                },
                                Colour::Black => { 
                                    let p = self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }];
                                    if self.white_mods.contains(&Mods::TripleCheck(p)) && self.board.get_game_state() == GameState::Check {
                                        if self.board.board.iter_mut().any(|(k, v)| v == &mut PieceType::King(Colour::Black) && p.valid_destinations(&Position { file: pos_x as u8, rank: pos_y as u8 }).contains(&k)) {
                                            self.triple_check_counter = (self.triple_check_counter.0, self.triple_check_counter.1 + 1);
                                            if self.triple_check_counter.1 >= 3 {
                                                self.end_game(Some(Colour::White));
                                            }
                                        }
                                    }
                                    if (self.white_mods.contains(&Mods::Atomic(p))) && !sniper && taking_move {
                                        self.taken_white_pieces.push(self.board.board[&Position { file: pos_x as u8, rank: pos_y as u8 }]);
                                        self.board.board.remove(&Position { file: pos_x as u8, rank: pos_y as u8 });
                                    } 
                                }
                            }
                        }
                    } else {
                        if self.selected_pos.1 == 9 { 
                            self.board.board.insert(Position { file: pos_x as u8, rank: pos_y as u8 }, self.taken_black_pieces[self.selected_pos.0 as usize].type_as_colour(Colour::White));
                            self.taken_black_pieces.remove(self.selected_pos.0 as usize); 
                            self.board.get_game_state();
                            self.board.active_color = Colour::Black;
                        }
                        else if self.selected_pos.1 == 10 { 
                            self.board.board.insert(Position { file: pos_x as u8, rank: pos_y as u8 }, self.taken_white_pieces[self.selected_pos.0 as usize].type_as_colour(Colour::Black));
                            self.taken_white_pieces.remove(self.selected_pos.0 as usize); 
                            self.board.get_game_state();
                            self.board.active_color = Colour::White;
                        }
                    }

                    self.selected_pos = (0, 0);
                    self.highlighted_pos = Vec::new();
                    return;
                }
                let mut real_board_but_copy = self.board.board.clone();
                for (k, v) in real_board_but_copy.iter_mut() {
                    let knig = vec![Position::from_string("e4".to_owned()).unwrap(), Position::from_string("e5".to_owned()).unwrap(), Position::from_string("d4".to_owned()).unwrap(), Position::from_string("d5".to_owned()).unwrap()];
                    if v == &mut PieceType::King(Colour::White) && knig.contains(k) {
                        self.end_game(Some(Colour::White));
                    }
                    if v == &mut PieceType::King(Colour::Black) && knig.contains(k) {
                        self.end_game(Some(Colour::Black));
                    }
                }
                self.highlighted_pos = Vec::new();
                self.selected_pos = (pos_x as isize, pos_y as isize);
                if self.board.board.contains_key(&Position { file: pos_x as u8, rank: pos_y as u8 }) {
                    for mov in self.board.get_possible_moves(Position { file: pos_x as u8, rank: pos_y as u8 }.to_string()).unwrap() {
                        let _mov = Position::from_string(mov).unwrap();
                        self.highlighted_pos.push((_mov.file as isize, _mov.rank as isize));
                    }
                }
            }

            if x >= SCREEN_SIZE.0 * 0.25 - GRID_CELL_SIZE.0 as f32 && y >= (GRID_CELL_SIZE.1 * 9) as f32 {
                let pos_x = x - (SCREEN_SIZE.0 * 0.25 - GRID_CELL_SIZE.0 as f32);
                let pos_x = (pos_x / GRID_CELL_SIZE.0 as f32).ceil();
                let pos_y = y - ((GRID_CELL_SIZE.1 * 9) as f32);
                let pos_y = (pos_y / GRID_CELL_SIZE.0 as f32).ceil();

                if (pos_y == 1f32 && pos_x <= self.taken_black_pieces.len() as f32) || (pos_y == 2f32 && pos_x <= self.taken_white_pieces.len() as f32) {
                    self.selected_pos = ((pos_x - 1f32) as isize, pos_y as isize + 8);
                    match self.board.active_color {
                        Colour::Black => {
                            if self.selected_pos.1 == 10 && self.black_mods.contains(&Mods::CrazyHouse(self.taken_white_pieces[self.selected_pos.0 as usize])) {
                                for x in 1..9 {
                                    for y in 1..9 {
                                        if !self.board.board.contains_key(&Position { file: x as u8, rank: y as u8}) {
                                            self.highlighted_pos.push((x, y));
                                        }
                                    }
                                }
                            }
                        },
                        Colour::White => {
                            if self.selected_pos.1 == 9 && self.white_mods.contains(&Mods::CrazyHouse(self.taken_black_pieces[self.selected_pos.0 as usize])) {
                                for x in 1..9 {
                                    for y in 1..9 {
                                        if !self.board.board.contains_key(&Position { file: x as u8, rank: y as u8}) {
                                            self.highlighted_pos.push((x, y));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } 

            if x >= SCREEN_SIZE.0 * 0.75 && x <= SCREEN_SIZE.0 * 0.75 + (GRID_CELL_SIZE.0 * 4) as f32 && y >= (GRID_CELL_SIZE.1 * 3) as f32 && y < (GRID_CELL_SIZE.1 * 4) as f32 {
                let pos_x = x - (SCREEN_SIZE.0 * 0.75 as f32);
                let pos_x = (pos_x / GRID_CELL_SIZE.0 as f32).ceil();

                match pos_x {
                    1f32 => self.board.set_promotion("queen".to_string()),
                    2f32 => self.board.set_promotion("rook".to_string()),
                    3f32 => self.board.set_promotion("bishop".to_string()),
                    4f32 => self.board.set_promotion("knight".to_string()),
                    _ => panic!(),
                };
            }
        }
    }
}

trait Gets {
    fn colour(&self) -> Colour;
    fn type_as_colour(&self, col: Colour) -> PieceType;
}

impl Gets for PieceType {
    fn colour(&self) -> Colour {
        match self {
            PieceType::Bishop(_colour) => *_colour,
            PieceType::Rook(_colour) => *_colour,
            PieceType::Pawn(_colour) => *_colour,
            PieceType::Knight(_colour) => *_colour,
            PieceType::Queen(_colour) => *_colour,
            PieceType::King(_colour) => *_colour,
        }
    }

    fn type_as_colour(&self, col: Colour) -> PieceType {
        match self {
            PieceType::Bishop(_colour) => PieceType::Bishop(col),
            PieceType::Rook(_colour) => PieceType::Rook(col),
            PieceType::Pawn(_colour) => PieceType::Pawn(col),
            PieceType::Knight(_colour) => PieceType::Knight(col),
            PieceType::Queen(_colour) => PieceType::Queen(col),
            PieceType::King(_colour) => PieceType::King(col),
        }
    }
}

fn generate_mod(col: Colour, rng1: u32, rng2: u32) -> Mods {
    let piece = match rng1 % 100 {
        0..=33 => PieceType::Pawn(Colour::Black),
        34..=53 => PieceType::Bishop(Colour::Black),
        54..=73 => PieceType::Knight(Colour::Black),
        74..=89 => PieceType::Rook(Colour::Black),
        90..=99 => PieceType::Queen(Colour::Black),
        _ => PieceType::Pawn(Colour::Black)
    };
    let mut not_col;
    if col == Colour::Black {
        not_col = Colour::White;
    } else {
        not_col = Colour::Black;
    }

    match rng2 % 100 {
        0..=9 => Mods::KingOfTheHill,
        10..=27 => Mods::Atomic(piece.type_as_colour(col)),
        28..=45 => Mods::CrazyHouse(piece.type_as_colour(not_col)),
        46..=63 => Mods::Extinction(piece.type_as_colour(not_col)),
        64..=81 => Mods::Sniper(piece.type_as_colour(col)),
        82..=99 => Mods::TripleCheck(piece.type_as_colour(col)),
        _ => Mods::KingOfTheHill,
    }
}

fn int_to_pos_tuple(x: isize) -> (isize, isize) {
    let pos_x = &x % 8;
    let pos_y = ((x as f32 / 8.0).trunc()) as isize; 
    (pos_x + 1, 9 - (pos_y + 1))
}

pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./resources");

    let context_builder = ggez::ContextBuilder::new("chess", "felix")
        .add_resource_path(resource_dir)        // Import image files to GGEZ
        .window_setup(
            ggez::conf::WindowSetup::default()  
                .title("Chess")                // Set window title "Schack"
                .icon("/icon.ico")              // Set application icon
        )
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1) // Set window dimenstions
                .resizable(false)               // Fixate window size
        );
    let (contex, event_loop) = &mut context_builder.build()?;

    let state = &mut AppState::new(contex)?;
    event::run(contex, event_loop, state)       // Run window event loop
}