use crate::Pt2D;
use geom::Bounds;
use std::ops::{Index, IndexMut};
use plotters::prelude::*;

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

    fn min_max(&self) -> (f64, f64) {
        self.data.iter().cloned().fold((std::f64::MAX, -std::f64::MAX), |(min, max), x| (f64::min(min, x), f64::max(max, x)))
    }

    // The 2D diffusion equation is given by
    // d / dt phi (x, y, t) = kappa * (d² / dx² + d² / dy²) phi (x, y, t)
    // or phi(x, y, t + dt) = phi(x, y, t) +
    //     dt * kappa / dx² * ( phi(x+dx, t) + phi(x - dx, y, t) + phi(x, y + dy t) + phi(x, y - dy, t) - 4 * phi(x, y, t))
    pub fn diffuse(&mut self, kappa: f64, decay: f64, dx: f64, dt: f64) {
        assert!(1.0 - 4.0 * dt / (dx * dx) * kappa - dt * decay > 0.0);
        let cpy = self.clone();
        for x in 1..self.width - 1 {
            for y in 1..self.height - 1 {
                self[(x, y)] *= 1.0 - 4.0 * dt / (dx * dx) * kappa - dt * decay;
                self[(x, y)] += dt / (dx * dx) * kappa
                    * (cpy[(x + 1, y)] + cpy[(x - 1, y)] + cpy[(x, y + 1)] + cpy[(x, y - 1)]);
            }
        }
    }

    pub fn absorb(&mut self, min: f64) {
        self.data.iter_mut().for_each(|x| {
            if *x < min {
                *x = 0.0
            }
        });
        // let (min, max) = self.min_max();
        // println!("min = {}, max = {}", min, max);
    }

    pub fn add_sources(
        &mut self,
        walkers: &Vec<Pt2D>,
        bounds: &Bounds,
        dx: f64,
        dt: f64,
        mag_per_sec: f64,
    ) {
        for w in walkers {
            let x = ((w.x() - bounds.min_x) / dx).floor() as usize;
            let y = ((w.y() - bounds.min_y) / dx).floor() as usize;

            // println!("x = {}, y = {}, w.x() = {}, w.y() = {}", x, y, w.x(), w.y());


            self[(x, y)] += dt * mag_per_sec;
            // self[(x+1, y)] += dt * mag_per_sec;
            // self[(x-1, y)] += dt * mag_per_sec;
            // self[(x, y+1)] += dt * mag_per_sec;
            // self[(x, y-1)] += dt * mag_per_sec;
        }
    }

    pub fn draw(&self, min: f64, max: f64, fname: &str) {
        let root =
            BitMapBackend::new(fname, (self.width as u32, self.height as u32)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Virus by pedestrians", ("sans-serif", 10))
            .margin(5)
            .top_x_label_area_size(40)
            .y_label_area_size(40)
            .build_ranged(0i32..self.width as i32, 0i32..self.height as i32).unwrap();

        chart
            .configure_mesh()
            .x_labels(15)
            .y_labels(15)
            .x_label_offset(35)
            .y_label_offset(25)
            .disable_x_mesh()
            .disable_y_mesh()
            .label_style(("sans-serif", 10))
            .draw().unwrap();

        let plotting_area = chart.plotting_area();

        for x in 0..self.width {
            for y in 0..self.height {
                let c = self[(x, y)];
                if max - min == 0.0 {
                    // plotting_area.draw(Rectangle::new(
                    //     [(x, y), (x + 1, y + 1)],
                    //     HSLColor(
                    //         240.0 / 360.0 - 240.0 / 360.0 * (*v as f64 / 20.0),
                    //         0.7,
                    //         0.1 + 0.4 * *v as f64 / 20.0,
                    //     )
                    //     .filled()).unwrap());
                    plotting_area.draw_pixel((x as i32, y as i32), &WHITE).unwrap();
                } else {
                    // println!("{}, {}, {}", min, max, c);
                    plotting_area.draw_pixel((x as i32, y as i32), &HSLColor((c - min) / (max - min), 1.0, 0.5)).unwrap();
                }
            }
        }
    }

    pub fn draw_autoscale(&self, fname: &str) {
        let (min, max) = self.min_max();
        println!("min = {}, max = {}", min, max);
        self.draw(min, max, fname);
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
