use crate::{game, rend, styling};

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
                block_gap,
            },
    }: &styling::Settings,
    base_layer: &mut rend::Layer,
) {
    let quad = quad(
        styling.bg2,
        *game_x,
        *game_y,
        (block_size + block_gap) * game::BOARD_WIDTH as u32 + *block_gap,
        (block_size + block_gap) * game::BOARD_VISIBLE_HEIGHT as u32 + block_gap,
    );
    base_layer.set_quads(vec![quad]);
}

pub fn game_quads(
    styling::Settings {
        palette: styling,
        sizing:
            styling::Sizing {
                game_x,
                game_y,
                block_size,
                block_gap,
            },
    }: &styling::Settings,
    game: &game::Game,
    game_layer: &mut rend::Layer,
) {
    let mut quads = Vec::with_capacity(game::TOTAL_BLOCKS as usize);
    let mut cx = *game_x;
    let mut cy = *game_y;

    // skip first four non-visible lines
    for line in game::VISIBLE_START..game::BOARD_HEIGHT {
        cy += block_gap;
        for b in game.block_iter(line) {
            cx += block_gap;
            let mut quad = quad(
                styling.colour_block(b.map(|(b, _)| b)),
                cx,
                cy,
                *block_size,
                *block_size,
            );
            if b.is_some_and(|(_, g)| g) {
                quad.colour.r *= 0.65;
                quad.colour.g *= 0.65;
                quad.colour.b *= 0.65;
            }
            quads.push(quad);

            cx += block_size;
        }
        cy += block_size;
        cx = *game_x;
    }

    game_layer.set_quads(quads);
}
