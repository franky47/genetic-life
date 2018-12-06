extern crate js_sys;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

fn rand_range(min: u8, max: u8) -> u8 {
  min + (js_sys::Math::random() * (max - min) as f64).round() as u8
}

// --

#[wasm_bindgen]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
  pub alive: bool,
  r: u8,
  g: u8,
  b: u8,
  age: u32,
  life_expectancy: u32,
}

#[wasm_bindgen]
impl Cell {
  pub fn get_color_hex(&self) -> String {
    if !self.alive {
      return String::from("#000000");
    }
    format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
  }
}

impl Cell {
  pub fn new() -> Cell {
    let mut c = Cell {
      ..Default::default()
    };
    c.reset();
    c
  }

  pub fn reset(&mut self) {
    self.alive = js_sys::Math::random() > 0.5;
    self.r = rand_range(0, 255);
    self.g = rand_range(0, 255);
    self.b = rand_range(0, 255);
    self.age = 0;
    self.decode_genome();
  }

  pub fn give_birth(&mut self, parents: [&Cell; 3]) {
    let was_alive = self.alive;
    self.r = mix_genes([&parents[0].r, &parents[1].r, &parents[2].r]);
    self.g = mix_genes([&parents[0].g, &parents[1].g, &parents[2].g]);
    self.b = mix_genes([&parents[0].b, &parents[1].b, &parents[2].b]);
    self.alive = match self.get_color() {
      0x00000 => false, // Unviable
      0xfffff => false, // Unviable
      _ => true,
    };

    if self.alive && !was_alive {
      self.age = 0;
      self.decode_genome();
    }
  }

  pub fn live_on(&mut self) {
    self.age += 1;
    if self.age > (self.life_expectancy as f64 * 0.75).floor() as u32
      && js_sys::Math::random() > 0.99
    {
      // Introduce mutation
      fn mutate(x: u8) -> u8 {
        x ^ (rand_range(0, 255) & rand_range(0, 255))
      }

      let gene = rand_range(0, 2);
      match gene {
        0 => self.r = mutate(self.r),
        1 => self.g = mutate(self.g),
        2 => self.b = mutate(self.b),
        _ => (),
      };
    }

    self.decode_genome();
    if self.age > self.life_expectancy {
      self.kill();
    }
  }

  pub fn kill(&mut self) {
    self.alive = false;
    self.age = 0;
    self.life_expectancy = 0;
    self.r = 0;
    self.g = 0;
    self.b = 0;
  }

  pub fn get_color(&self) -> u32 {
    (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
  }

  // --

  fn decode_genome(&mut self) {
    fn pick_gene_bit(gene: u8, n: u8) -> bool {
      (gene & (1 << n)) >> n == 1
    }

    // Disease genes
    let disease_r = pick_gene_bit(self.r, 4) || pick_gene_bit(self.r, 1);
    let disease_g = pick_gene_bit(self.g, 4) || pick_gene_bit(self.g, 1);
    let disease_b = pick_gene_bit(self.b, 4) || pick_gene_bit(self.b, 1);

    // Health genes
    let health_r = pick_gene_bit(self.r, 6) || pick_gene_bit(self.r, 3);
    let health_g = pick_gene_bit(self.g, 6) || pick_gene_bit(self.g, 3);
    let health_b = pick_gene_bit(self.b, 6) || pick_gene_bit(self.b, 3);

    // Dominant/recessive behaviour gene
    let dom_r = self.r & (1 << 5) != 0;
    let dom_g = self.g & (1 << 5) != 0;
    let dom_b = self.b & (1 << 5) != 0;

    fn rand_probability(dom: bool) -> f64 {
      let lo = if dom { 0.6 } else { 0.0 };
      let hi = if dom { 1.0 } else { 0.7 };
      42. * (js_sys::Math::random() * (hi - lo) + lo)
    }

    let prob_r = rand_probability(dom_r);
    let prob_g = rand_probability(dom_g);
    let prob_b = rand_probability(dom_b);

    fn factor(health: bool, disease: bool, prob: f64) -> i32 {
      let h = if health { 1 } else { 0 };
      let d = if disease { 1 } else { 0 };
      (prob * (h - d) as f64).round() as i32
    }

    // Reduce the life of low-brightness cells
    let b = (0.2126 * self.r as f64 + 0.7152 * self.g as f64 + 0.0722 * self.b as f64) / 255.;
    fn sigmoid(x: f64) -> f64 {
      let a = 8.;
      let b = 6.;
      (a * x + b).tanh() * 0.5 + 0.5
    }
    let average_life_expectancy = (sigmoid(b) * 500.).round() as i32;

    self.life_expectancy = std::cmp::max(
      0,
      average_life_expectancy
        + factor(health_r, disease_r, prob_r)
        + factor(health_g, disease_g, prob_g)
        + factor(health_b, disease_b, prob_b),
    ) as u32;
  }
}

// --

fn mix_genes(genes: [&u8; 3]) -> u8 {
  // We need to imagine three parents mixing their genes to produce a child.
  // Rather than having the CGTA DNA mixing system that is valid for two parents,
  // we split the genes of the parents threefold (namely a, b, c), and pick one
  // gene segment from each parent to compose the child's.
  // As there are 5 possible permutations, we randomly select which will occur.
  // Gene mutation cuts:
  //        a   b   c
  // MSB [---|----|-] LSB
  //         hi	  lo  here: lo=1, hi=5

  let l: u8 = rand_range(0, 3);
  let h: u8 = rand_range(4, 7);

  let a_mask: u8 = 0xff << h;
  let c_mask: u8 = !(0xff << l);
  let b_mask: u8 = !(a_mask | c_mask);

  let mutation_type: u8 = rand_range(0, 5);
  match mutation_type {
    0 => (genes[0] & a_mask) | (genes[1] & b_mask) | (genes[2] & c_mask),
    1 => (genes[0] & a_mask) | (genes[2] & b_mask) | (genes[1] & c_mask),
    2 => (genes[1] & a_mask) | (genes[0] & b_mask) | (genes[2] & c_mask),
    3 => (genes[1] & a_mask) | (genes[2] & b_mask) | (genes[0] & c_mask),
    4 => (genes[2] & a_mask) | (genes[0] & b_mask) | (genes[1] & c_mask),
    5 => (genes[2] & a_mask) | (genes[1] & b_mask) | (genes[0] & c_mask),
    _ => 0,
  }
}
