use crate::{game, rend, styling};

// TODO: create drawing context

fn quad(colour: styling::Colour, x: u32, y: u32, width: u32, height: u32) -> rend::Quad {
    rend::Quad {
        colour,
        x,
        y,
        width,
        height,
    }
}

pub fn base_quads(
    styling::Settings {
        palette: styling,
        sizing:
            styling::Sizing {
                game_x,
                game_y,
                block_size,
            },
    }: &styling::Settings,
    base_layer: &mut rend::QuadLayer,
) {
    let quad = quad(
        styling.bg2,
        *game_x,
        *game_y,
        block_size * game::BOARD_WIDTH as u32,
        block_size * game::BOARD_VISIBLE_HEIGHT as u32,
    );
    base_layer.set_quads(vec![quad]);
}

pub fn game_quads(
    settings: &styling::Settings,
    game: &game::Game,
    game_layer: &mut rend::QuadLayer,
) {
    QDraw {
        settings,
        game,
        game_layer,
        quads: Vec::new(),
    }
    .draw_game();
}

struct QDraw<'a> {
    settings: &'a styling::Settings,
    game: &'a game::Game,
    game_layer: &'a mut rend::QuadLayer,
    quads: Vec<super::rend::Quad>,
}

impl QDraw<'_> {
    pub fn draw_game(mut self) {
        self.draw_board();
        self.draw_next();
        self.draw_held();
        self.draw_mino(self.game.ghost(), |c| c * 0.3);
        self.draw_mino(self.game.mino(), |c| c);

        self.game_layer.set_quads(self.quads);
    }

    fn draw_board(&mut self) {
        let styling::Settings {
            palette,
            sizing:
                styling::Sizing {
                    game_x,
                    game_y,
                    block_size,
                },
        } = self.settings;
        self.quads.reserve(game::TOTAL_BLOCKS as usize);
        let mut cx = *game_x;
        let mut cy = *game_y;

        // skip first four non-visible lines
        for line in game::VISIBLE_START..game::BOARD_HEIGHT {
            for b in self.game.blocks(line) {
                self.push_square(palette.colour_block(b), cx, cy);
                cx += block_size;
            }
            cy += block_size;
            cx = *game_x;
        }
    }

    fn draw_next(&mut self) {
        let styling::Settings {
            palette,
            sizing:
                styling::Sizing {
                    game_x,
                    game_y,
                    block_size,
                },
        } = self.settings;
        let next_x = *game_x + block_size * game::BOARD_WIDTH as u32 + block_size / 2;
        let mut next_y = *game_y;

        for &b in &self.game.bag().minos[..5] {
            for game::Point { x, y } in b.points(Default::default()) {
                self.push_square(
                    palette.colour_block(Some(b)),
                    next_x + x as u32 * block_size,
                    next_y + y as u32 * block_size,
                );
            }
            // 2.5 * block_size
            next_y += block_size * 2 + block_size / 2;
        }
    }

    fn draw_held(&mut self) {
        let styling::Settings {
            palette,
            sizing:
                styling::Sizing {
                    game_x,
                    game_y,
                    block_size,
                },
        } = self.settings;
        // gmae_x - 4.5 * block_size
        let Some(held_x) = game_x.checked_sub(block_size * 4 + block_size / 2) else {
            return;
        };
        let held_y = game_y;
        let Some(held) = self.game.bag().held else {
            return;
        };
        for game::Point { x, y } in held.points(Default::default()) {
            self.push_square(
                palette.colour_block(Some(held)),
                held_x + x as u32 * block_size,
                held_y + y as u32 * block_size,
            );
        }
    }

    fn draw_mino(&mut self, mino: game::Mino, colour: impl Fn(styling::Colour) -> styling::Colour) {
        let styling::Settings {
            palette,
            sizing:
                styling::Sizing {
                    game_x,
                    game_y,
                    block_size,
                },
        } = self.settings;
        let Some(points) = mino.real_points() else {
            return;
        };
        for p in points {
            self.push_square(
                colour(palette.colour_block(Some(mino.block))),
                game_x + p.x as u32 * block_size,
                game_y + p.y.saturating_sub(game::VISIBLE_START) as u32 * block_size,
            );
        }
    }
    fn push_square(&mut self, colour: styling::Colour, x: u32, y: u32) {
        let s = self.settings.sizing.block_size;
        self.quads.push(quad(colour, x, y, s, s));
        // borders
        let tint = styling::Colour {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        };
        if s != 0 {
            for quad in [
                quad(tint, x, y, s, 1),         // up
                quad(tint, x, y, 1, s),         // left
                quad(tint, x, y + s - 1, s, 1), // down
                quad(tint, x + s - 1, y, 1, s), // right
            ] {
                self.quads.push(quad);
            }
        }
    }
}
