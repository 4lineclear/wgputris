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
    game: &game::Game,
    base_layer: &mut rend::Layer,
    push: bool,
) {
    let quad = quad(
        styling.bg2,
        *game_x,
        *game_y,
        (block_size + block_gap) * game.board().line(0).blocks().len() as u32 + *block_gap,
        (block_size + block_gap) * game.board().visible().len() as u32 + block_gap,
    );
    if push {
        base_layer.push(quad);
    } else {
        base_layer.set(0, quad);
    }
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
    push: bool,
) {
    let mut i = 0;
    let mut cx = *game_x;
    let mut cy = *game_y;
    for line in game.board().visible() {
        cy += block_gap;
        for &b in line.blocks() {
            cx += block_gap;
            let quad = quad(styling.colour_block(b), cx, cy, *block_size, *block_size);
            if push {
                game_layer.push(quad);
            } else {
                game_layer.set(i, quad);
            }
            cx += block_size;
            i += 1;
        }
        cy += block_size;
        cx = *game_x;
    }
}
