pub mod cell;
mod utils;

use crate::cell::Cell;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub struct Universe {
  width: u32,
  height: u32,
  cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
  pub fn new(width: u32, height: u32) -> Universe {
    utils::set_panic_hook();
    let cells: Vec<Cell> = (0..width * height).map(|_| Cell::new()).collect();

    Universe {
      width,
      height,
      cells,
    }
  }

  pub fn reset(&mut self) {
    for cell in &mut self.cells {
      cell.reset();
    }
  }

  pub fn tick(&mut self) {
    let mut next = self.cells.clone();

    for row in 0..self.height {
      for col in 0..self.width {
        let idx = self.get_index(row, col);
        let live_neighbours = self.live_neighbours(row, col);
        let num_live_neighbours = live_neighbours.len();

        if self.cells[idx].alive {
          // Fewer than 2 live neighbours => death
          if num_live_neighbours < 2 {
            next[idx].kill(); // Starvation
          }
          if num_live_neighbours == 2 || num_live_neighbours == 3 {
            // Live on
            next[idx].live_on();
          }
          if num_live_neighbours > 3 {
            next[idx].kill(); // Overpopulation
          }
        } else {
          if num_live_neighbours == 3 {
            let parents = [live_neighbours[0], live_neighbours[1], live_neighbours[2]];
            next[idx].give_birth(parents);
          }
        }
      }
    }
    self.cells = next;
  }

  pub fn cell(&self, row: u32, column: u32) -> Cell {
    let idx = self.get_index(row, column);
    self.cells[idx]
  }
}

// --

impl Universe {
  fn get_index(&self, row: u32, column: u32) -> usize {
    (row * self.width + column) as usize
  }

  fn live_neighbours(&self, row: u32, column: u32) -> Vec<&Cell> {
    let mut neighbours: Vec<&Cell> = Vec::new();
    for delta_row in [self.height - 1, 0, 1].iter().cloned() {
      for delta_col in [self.width - 1, 0, 1].iter().cloned() {
        if delta_row == 0 && delta_col == 0 {
          continue; // don't count self
        }

        let neighbour_row = (row + delta_row) % self.height;
        let neighbour_col = (column + delta_col) % self.width;
        let idx = self.get_index(neighbour_row, neighbour_col);
        if self.cells[idx].alive {
          neighbours.push(&self.cells[idx]);
        }
      }
    }
    neighbours
  }
}
