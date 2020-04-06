// This code is inspired by the Palabos source code: www.palabos.org
use geom::Pt2D;
use std::cmp::Ordering;
use ezgui::{Color};
use std::collections::BinaryHeap;

trait ScalarFunction {
    fn compute(&self, x: f64) -> f64;
}

struct LinearFunction {
    p1: Pt2D,
    p2: Pt2D,
}

impl ScalarFunction for LinearFunction {
    fn compute(&self, x: f64) -> f64 {
        ((self.p2.y() - self.p1.y()) * x + self.p2.x() * self.p1.y() - self.p1.x() * self.p2.y())
            / (self.p2.x() - self.p1.x())
    }
}

struct PowerLawFunction {
    p1: Pt2D,
    p2: Pt2D,
    b: f64,
}

impl PowerLawFunction {
    fn new(p1: Pt2D, p2: Pt2D, b: f64) -> Self {
        PowerLawFunction { p1, p2, b }
    }
}

impl ScalarFunction for PowerLawFunction {
    fn compute(&self, x: f64) -> f64 {
        ((self.p2.y() - self.p1.y()) * x.powf(self.b) + self.p2.x() * self.p1.y()
            - self.p1.x() * self.p2.y())
            / (self.p2.x() - self.p1.x())
    }
}

#[derive(PartialEq, PartialOrd)]
struct Piece {
    closed_begin: f64,
    open_end: f64,
}

impl Piece {
    fn new(closed_begin: f64, open_end: f64) -> Self {
        Piece {
            closed_begin,
            open_end,
        }
    }
}

impl Ord for Piece {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for Piece {}

impl Piece {
    fn contains(&self, val: f64) -> bool {
        val >= self.closed_begin && val < self.open_end
    }
}

struct Function {
    piece: Piece,
    function: Box<dyn ScalarFunction>,
}

impl Function {
    fn new(piece: Piece, function: Box<dyn ScalarFunction>) -> Self {
        Function { piece, function }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.piece == other.piece
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.piece.partial_cmp(&other.piece)
    }
}

impl Eq for Function {}

impl Ord for Function {
    fn cmp(&self, other: &Self) -> Ordering {
        self.piece.cmp(&other.piece)
    }
}

struct PiecewiseFunction {
    functions: BinaryHeap<Function>,
}

impl PiecewiseFunction {
    fn new() -> Self {
        PiecewiseFunction {
            functions: BinaryHeap::new(),
        }
    }

    fn check_non_overlapping_piece(&self, piece: &Piece) -> bool {
        !self
            .functions
            .iter()
            .any(|f| f.piece.contains(piece.closed_begin) || f.piece.contains(piece.open_end))
    }

    fn add_piece(mut self, piece: Piece, foo: Box<dyn ScalarFunction>) -> Result<Self, String> {
        if self.check_non_overlapping_piece(&piece) {
            return Err(String::from("Pieces are overlapping."));
        }
        self.functions.push(Function::new(piece, foo));
        Ok(self)
    }
}

impl ScalarFunction for PiecewiseFunction {
    fn compute(&self, x: f64) -> f64 {
        // TODO should adapt this code for binary heap. Not using at all the sorting.
        for Function { piece, function } in self.functions.iter() {
            if piece.contains(x) {
                return function.compute(x);
            }
        }
        std::f64::NAN
    }
}


pub struct Colormap {
    red: PiecewiseFunction,
    green: PiecewiseFunction,
    blue: PiecewiseFunction,
}

impl Colormap {
    fn new(red: PiecewiseFunction, green: PiecewiseFunction, blue: PiecewiseFunction) -> Self {
        Colormap{red, green, blue}
    }

    fn in_range(x: f64) -> Option<f64> {
        if x >= 0.0 && x <= 1.0 {
            return Some(x);
        }
        None
    }

    pub fn rgb_f(&self, x: f64) -> Color {
        assert!(x >= 0.0 && x <= 1.0);
        Color::rgb_f(
            Colormap::in_range(self.red.compute(x)).unwrap() as f32,
            Colormap::in_range(self.green.compute(x)).unwrap() as f32,
            Colormap::in_range(self.blue.compute(x)).unwrap() as f32,
        )
    }
}

pub fn earth() -> Colormap {
    Colormap::new(generate_earth_red().unwrap(), generate_earth_green().unwrap(), generate_earth_blue().unwrap())
}

fn generate_earth_red() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p0, p1),
                Pt2D::new(0.0, 0.8),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p1, p2),
                Pt2D::new(0.8, 0.9),
                0.9,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p2, p3),
                Pt2D::new(0.9, 1.0),
                0.2,
            )),
        )
}

fn generate_earth_green() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p0, p1),
                Pt2D::new(0.0, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p1, p2),
                Pt2D::new(0.5, 0.9),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p2, p3),
                Pt2D::new(0.9, 1.0),
                0.2,
            )),
        )
}

fn generate_earth_blue() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p0, p1),
                Pt2D::new(0.0, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p1, p2),
                Pt2D::new(0.5, 0.7),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Pt2D::new(p2, p3),
                Pt2D::new(0.7, 1.0),
                0.2,
            )),
        )
}
