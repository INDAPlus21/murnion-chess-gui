/**
 * Chess GUI template.
 * Author: Eskil Queseth <eskilq@kth.se>, Viola Söderlund <violaso@kth.se>
 * Last updated: 2020-09-20
 */

use ggez::event;
use ggez::graphics::{self, DrawParam, Color, DrawMode};
use ggez::{Context, GameResult};
use std::path;
use eliasfl_chess::{Game, GameState, Color as Colour, Piece as PieceType};
use ggez::event::{MouseButton};
use std::collections::HashMap;

/// A chess board is 8x8 tiles.
const GRID_SIZE: (i16, i16) = (8, 8);
/// Sutible size of each tile.
const GRID_CELL_SIZE: (i16, i16) = (45, 45);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32 * 1.5,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

// GUI Color representations
const BLACK: Color = Color::new(228.0/255.0, 196.0/255.0, 108.0/255.0, 1.0);
const WHITE: Color = Color::new(188.0/255.0, 140.0/255.0, 76.0/255.0, 1.0);

/// GUI logic and event implementation structure. 
struct AppState {
    sprites: HashMap<PieceType, graphics::Image>,
    board: Game
    // Save piece positions, which tiles has been clicked, current colour, etc...
}

impl AppState {
    /// Initialise new application, i.e. initialise new game and load resources.
    fn new(ctx: &mut Context) -> GameResult<AppState> {
        let sprites = AppState::load_sprites();
        let board = Game::new();

        let state = AppState {
            sprites: sprites
                .iter()
                .map(|_sprite| {
                    (_sprite.0, graphics::Image::new(ctx, _sprite.1.clone()).unwrap())
                })
                .collect::<HashMap<PieceType, graphics::Image>>(),
            board
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
}

/// Implement each stage of the application event loop. 
impl event::EventHandler for AppState {

    /// For updating game logic, which front-end doesn't handle.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    /// Draw interface, i.e. draw game board
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear interface with gray background colour
        graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());

        // create text representation
        let state_text = graphics::Text::new(
                graphics::TextFragment::from(format!("Game is {:?}.", self.board.get_game_state())
            )
            .scale(graphics::Scale { x: 20.0, y: 20.0 }));

        // get size of text
        let text_dimensions = state_text.dimensions(ctx);
        // create background rectangle with white coulouring
        let background_box = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(),
            graphics::Rect::new(SCREEN_SIZE.0 * 2f32/3f32,
                                0f32,
                                SCREEN_SIZE.0 * 0.5, SCREEN_SIZE.1),
                                [1.0, 1.0, 1.0, 1.0].into()
        )?;

        // draw background
        graphics::draw(ctx, &background_box, DrawParam::default());

        // draw tiles
        for i in 0..64 {
            let rectangle = graphics::Mesh::new_rectangle(ctx, 
                graphics::DrawMode::fill(), 
                graphics::Rect::new_i32(
                    i % 8 * GRID_CELL_SIZE.0 as i32,
                    i / 8 * GRID_CELL_SIZE.1 as i32,
                    GRID_CELL_SIZE.0 as i32,
                    GRID_CELL_SIZE.1 as i32,
                ), match i % 2 {
                    0 => match i / 8 {
                        _row if _row % 2 == 0 => WHITE,
                        _ => BLACK
                    },
                    _ => match i / 8 {
                        _row if _row % 2 == 0 => BLACK,
                        _ => WHITE
                    }
                })?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ));
        }

        for (pos, val) in self.board.board.iter() {
            graphics::draw(ctx, &self.sprites[val], (ggez::mint::Point2 { x: (pos.file - 1) as f32 * GRID_CELL_SIZE.0 as f32, y: (pos.rank - 1) as f32 * GRID_CELL_SIZE.1 as f32 }, ));
        }

        // draw text with dark gray colouring and center position
        graphics::draw(ctx, &state_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
            .dest(ggez::mint::Point2 {
                x: SCREEN_SIZE.0 * 2f32/3f32,
                y: 0f32,
            }));

        // render updated graphics
        graphics::present(ctx)?;

        Ok(())
    }

    /// Update game on mouse click
    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            /* check click position and update board accordingly */
        }
    }
}

pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./resources");

    let context_builder = ggez::ContextBuilder::new("schack", "eskil")
        .add_resource_path(resource_dir)        // Import image files to GGEZ
        .window_setup(
            ggez::conf::WindowSetup::default()  
                .title("Schack")                // Set window title "Schack"
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