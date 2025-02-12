use crate::game;

#[derive(Debug)]
pub struct Settings {
    pub styling: Styling,
    pub sizing: Sizing,
}

// TODO: move to using textures for blocks
#[derive(Debug)]
pub struct Styling {
    pub fg: Colour,
    pub bg: Colour,
    /// empty block
    e: Colour,
    i: Colour,
    t: Colour,
    o: Colour,
    l: Colour,
    j: Colour,
    s: Colour,
    z: Colour,
}

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Colour {
    pub fn rgb(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
    pub fn rgba(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Styling {
    pub fn colour_block(&self, block: Option<game::Block>) -> Colour {
        match block {
            Some(game::Block::I) => self.i,
            Some(game::Block::T) => self.t,
            Some(game::Block::O) => self.o,
            Some(game::Block::L) => self.l,
            Some(game::Block::J) => self.j,
            Some(game::Block::S) => self.s,
            Some(game::Block::Z) => self.z,
            None => self.e,
        }
    }
}

#[derive(Debug)]
pub struct Sizing {
    pub game_x: u32,
    pub game_y: u32,
    pub block_size: u32,
    pub block_gap: u32,
}

impl Sizing {
    pub fn resize(&mut self, game: &game::Game, width: u32, height: u32) {
        let n_wide = game.board().line(0usize).blocks().len() as u32;
        let n_tall = game.board().visible().len() as u32;
        let board_width = (self.block_gap + self.block_size) * n_wide;
        let board_height = (self.block_gap + self.block_size) * n_tall;
        self.game_x = (width / 2).saturating_sub(board_width / 2);
        self.game_y = (height / 2).saturating_sub(board_height / 2);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            styling: Styling {
                fg: colours::WHITE,
                bg: colours::WHITE,
                e: colours::SILVER,
                i: colours::WHITE,
                t: colours::WHITE,
                o: colours::WHITE,
                l: colours::WHITE,
                j: colours::WHITE,
                s: colours::WHITE,
                z: colours::WHITE,
            },
            sizing: Sizing {
                game_x: 0,
                game_y: 0,
                block_size: 30,
                block_gap: 5,
            },
        }
    }
}

pub mod colours {
    macro_rules! colours {
    ($($name:ident($($e:expr),*)),* $(,)?) => {
        $(
            pub const $name: super::Colour = colours!($($e),*);
        )*
    };
    ($r:expr, $g:expr, $b:expr) => {
        colours!($r, $g, $b, 1.0)
    };
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
         super::Colour {
            r: ($r / 255.0),
            g: ($g / 255.0),
            b: ($b / 255.0),
            a: $a,
        }
    };
}
    colours!(
        WHITE(255.0, 255.0, 255.0, 1.0),
        BLACK(0.0, 255.0, 255.0, 1.0),
        SILVER(191.0, 191.0, 191.0, 1.0),
        CYAN(0.0, 255.0, 255.0, 1.0),
    );
}
