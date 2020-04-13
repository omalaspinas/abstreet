use crate::{Pt2D};
use geom::Bounds;
use std::ops::{Index, IndexMut};


#[derive(Debug, Clone)]
pub struct Grid {
    data: Vec<f64>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, default: f64) -> Grid {
        Grid {
            data: std::iter::repeat(default).take(width * height).collect(),
            width,
            height,
        }
    }

    pub fn zero(width: usize, height: usize) -> Grid {
        Grid::new(width, height, 0.0)
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        // Row-major
        y * self.width + x
    }

    // safe way to get an element
    pub fn get(&self, x: usize, y: usize) -> Option<f64> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(self.data[self.idx(x, y)])
        }
    }

    // The 2D diffusion equation is given by
    // d / dt phi (x, y, t) = kappa * (d² / dx² + d² / dy²) phi (x, y, t)
    // or phi(x, y, t + dt) = phi(x, y, t) +
    //     dt * kappa / dx² * ( phi(x+dx, t) + phi(x - dx, y, t) + phi(x, y + dy t) + phi(x, y - dy, t) - 4 * phi(x, y, t))
    pub fn diffuse(&mut self, kappa: f64, dt: f64, dx: f64) {
        let mut cpy = self.clone();
        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                self[(x, y)] *= (-4.0 / (dx * dx) + dt);
                self[(x, y)] += dt / (dx * dx)
                    * (cpy[(x + 1, y)] + cpy[(x - 1, y)] + cpy[(x, y + 1)] + cpy[(x, y - 1)]);
            }
        }
    }

    pub fn crop(&mut self, min: f64, max: f64) {
        self.data.iter_mut().for_each(|x| {
            if *x > max {
                *x = max
            } else if *x < min {
                *x = min
            }
        });
    }

    pub fn add_sources(&mut self, walkers: &Vec<Pt2D>, bounds: &Bounds, dx:f64 , mag_per_sec: f64, dt: f64) {
        for w in walkers {
            let x = ((w.x() - bounds.min_x) * dx) as usize ;
            let y = ((w.y() - bounds.min_x) * dx) as usize ;

            self[(x, y)] += dt * mag_per_sec;
        }
    }
}

// out ofbounds may occur here
impl Index<(usize, usize)> for Grid {
    type Output = f64;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        &self.data[self.idx(x, y)]
    }
}

// out ofbounds may occur here
impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let idx = self.idx(x, y);
        &mut self.data[idx]
    }
}
