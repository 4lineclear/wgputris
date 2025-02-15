use crate::game;

#[derive(Debug)]
pub struct Settings {
    pub palette: Palette,
    pub sizing: Sizing,
}

// TODO: move to using textures for blocks
#[derive(Debug)]
pub struct Palette {
    pub fg: Colour,
    pub bg: Colour,
    pub fg2: Colour,
    pub bg2: Colour,
    /// empty block
    pub e: Colour,
    pub i: Colour,
    pub j: Colour,
    pub l: Colour,
    pub o: Colour,
    pub s: Colour,
    pub t: Colour,
    pub z: Colour,
}

#[derive(Debug, Default, Clone, Copy)]
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

impl Palette {
    pub fn colour_block(&self, block: Option<game::Block>) -> Colour {
        match block {
            Some(game::Block::I) => self.i,
            Some(game::Block::L) => self.l,
            Some(game::Block::J) => self.j,
            Some(game::Block::O) => self.o,
            Some(game::Block::S) => self.s,
            Some(game::Block::T) => self.t,
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
            sizing: Sizing {
                game_x: 0,
                game_y: 0,
                block_size: 30,
                block_gap: 2,
            },
            palette: dark_light::detect()
                .is_ok_and(|m| m == dark_light::Mode::Dark)
                .then(Palette::dark)
                .unwrap_or_else(Palette::light),
        }
    }
}

impl Palette {
    pub fn light() -> Self {
        Palette {
            fg: colours::BLACK,
            bg: colours::WHITE,
            fg2: colours::OFF_BLACK,
            bg2: colours::SOFT_WHITE,
            e: colours::SILVER,
            i: colours::CYAN,
            j: colours::BLUE,
            l: colours::ORANGE,
            o: colours::YELLOW,
            s: colours::GREEN,
            t: colours::PURPLE,
            z: colours::RED,
        }
    }
    pub fn dark() -> Self {
        let palette = Palette::light();
        Palette {
            fg: palette.bg,
            bg: palette.fg,
            fg2: palette.bg2,
            bg2: palette.fg2,
            ..palette
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
        // Standard colours
        WHITE(230.0, 230.0, 230.0, 1.0),
        BLACK(30.0, 30.0, 30.0, 1.0),
        SOFT_WHITE(200.0, 200.0, 200.0, 1.0),
        OFF_BLACK(50.0, 50.0, 50.0, 1.0),
        // block colours
        SILVER(160.0, 160.0, 160.0, 1.0),
        CYAN(0.0, 255.0, 255.0, 1.0),
        BLUE(0.0, 0.0, 255.0, 1.0),
        ORANGE(255.0, 165.0, 0.0, 1.0),
        YELLOW(255.0, 255.0, 0.0, 1.0),
        GREEN(0.0, 255.0, 0.0, 1.0),
        PURPLE(160.0, 32.0, 240.0, 1.0),
        RED(255.0, 0.0, 0.0, 1.0),
    );
}
