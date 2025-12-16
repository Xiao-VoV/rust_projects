use piston_window::types::Color;
use piston_window::{Context, G2d};
use rand::rand_core::block;
use std::collections::LinkedList;
use std::sync::mpsc::RecvTimeoutError;

use crate::draw::draw_block;

const SNAKE_COLOR: Color = [0.0, 0.8, 0.0, 1.0]; //RGBA
#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    up,
    down,
    left,
    right,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::up => Direction::down,
            Direction::down => Direction::up,
            Direction::left => Direction::right,
            Direction::right => Direction::left,
        }
    }
}

#[derive(Clone, Debug)]
struct Block {
    x: i32,
    y: i32,
}

pub struct Snake {
    direction: Direction,
    body: LinkedList<Block>,
    tail: Option<Block>,
}

impl Snake {
    pub fn new(x: i32, y: i32) -> Self {
        let mut body: LinkedList<Block> = LinkedList::new();

        body.push_back(Block { x: x + 2, y: y });
        body.push_back(Block { x: x + 1, y: y });
        body.push_back(Block { x: x, y: y });

        Snake {
            direction: Direction::right,
            body,
            tail: None,
        }
    }

    pub fn draw(&self, con: &Context, g: &mut G2d) {
        for block in &self.body {
            draw_block(SNAKE_COLOR, block.x, block.y, con, g);
        }
    }

    pub fn head_position(&self) -> (i32, i32) {
        let head_block = self.body.front().unwrap();
        (head_block.x, head_block.y)
    }
    pub fn move_forward(&mut self, dir: Option<Direction>) {
        match dir {
            Some(d) => self.direction = d,
            None => (),
        }

        let (last_x, last_y) = self.head_position();

        let new_block = match self.direction {
            Direction::up => Block {
                x: last_x,
                y: last_y - 1,
            },
            Direction::down => Block {
                x: last_x,
                y: last_y + 1,
            },
            Direction::left => Block {
                x: last_x - 1,
                y: last_y,
            },
            Direction::right => Block {
                x: last_x + 1,
                y: last_y,
            },
        };
        self.body.push_front(new_block);
        let remove_block = self.body.pop_back().unwrap();
        self.tail = Some(remove_block);
    }

    pub fn head_direction(&self) -> Direction {
        self.direction
    }

    pub fn next_head(&self, dir: Option<Direction>) -> (i32, i32) {
        let (head_x, head_y) = self.head_position();

        let mut moving_dir = self.direction;

        match dir {
            Some(d) => moving_dir = d,
            None => {}
        }

        match moving_dir {
            Direction::up => (head_x, head_y - 1),
            Direction::down => (head_x, head_y + 1),
            Direction::left => (head_x - 1, head_y),
            Direction::right => (head_x + 1, head_y),
        }
    }

    pub fn restore_tail(&mut self) {
        let blk = self.tail.clone().unwrap();
        self.body.push_back(blk);
    }

    pub fn overlap_tail(&self, x: i32, y: i32) -> bool {
        let mut ch = 0;
        for block in &self.body {
            if x == block.x && y == block.y {
                return true;
            }
            ch += 1;
            if ch == self.body.len() - 1 {
                break;
            }
        }
        return false;
    }
}
