use std::collections::{HashMap, HashSet};

use winit::event::KeyEvent;

use crate::Action;

#[derive(Debug, Clone)]
pub struct KeyStore {
    keys: HashMap<Key, Action>,
    pressed: HashSet<Key>,
}

impl Default for KeyStore {
    fn default() -> Self {
        use winit::keyboard::KeyCode;
        use Action::*;
        let keys = [
            (KeyCode::KeyC, Hold),
            (KeyCode::Space, Place),
            (KeyCode::ArrowUp, Rotate180),
            (KeyCode::KeyZ, RotateLeft),
            (KeyCode::KeyX, RotateRight),
            (KeyCode::ArrowRight, MoveRight),
            (KeyCode::ArrowLeft, MoveLeft),
            (KeyCode::ArrowDown, MoveDown),
            (KeyCode::Escape, Exit),
        ];
        Self {
            keys: keys.into_iter().map(|(kc, a)| (Key::Code(kc), a)).collect(),
            pressed: Default::default(),
        }
    }
}

impl KeyStore {
    pub fn register_key(&mut self, key: Key, action: Action) -> Option<Action> {
        self.keys.insert(key, action)
    }
    pub fn active(&self) -> bool {
        !self.pressed.is_empty()
    }
    pub fn apply_key(&mut self, key: Key, pressed: bool) -> Vec<Action> {
        let new_key = if pressed {
            self.keys
                .get(&key)
                .filter(|ea| ea.repeatable() || !self.pressed.contains(&key))
                .copied()
        } else {
            self.pressed.remove(&key);
            None
        };
        let actions = self.get_actions().chain(new_key).collect();
        if pressed {
            self.pressed.insert(key);
        }
        actions
    }
    pub fn get_actions<'a>(&'a self) -> impl Iterator<Item = Action> + 'a {
        self.pressed
            .iter()
            .filter_map(|k| self.keys.get(k).copied().filter(Action::repeatable))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Code(winit::keyboard::KeyCode),
    Numeric(u32),
}

impl Key {
    pub fn from_event(event: KeyEvent) -> Option<Self> {
        use winit::keyboard::NativeKeyCode;
        use winit::keyboard::PhysicalKey;
        match event.physical_key {
            PhysicalKey::Code(kc) => Some(Key::Code(kc)),
            PhysicalKey::Unidentified(nkc) => match nkc {
                NativeKeyCode::Unidentified => None,
                NativeKeyCode::Android(c) | NativeKeyCode::Xkb(c) => Some(Key::Numeric(c)),
                NativeKeyCode::MacOS(c) | NativeKeyCode::Windows(c) => Some(Key::Numeric(c as u32)),
            },
        }
    }
}
