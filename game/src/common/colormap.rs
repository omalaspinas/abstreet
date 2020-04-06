// This code is inspired by the Palabos source code: www.palabos.org
use ezgui::Color;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

struct Point2d {
    x: f64,
    y: f64,
}

impl Point2d {
    fn new(x: f64, y: f64) -> Self {
        Point2d { x, y }
    }
}

trait ScalarFunction {
    fn compute(&self, x: f64) -> f64;
}

struct LinearFunction {
    p1: Point2d,
    p2: Point2d,
}

impl LinearFunction {
    fn new(p1: Point2d, p2: Point2d) -> Self {
        LinearFunction { p1, p2 }
    }
}

impl ScalarFunction for LinearFunction {
    fn compute(&self, x: f64) -> f64 {
        assert!(self.p2.x != self.p1.x);
        ((self.p2.y - self.p1.y) * x + self.p2.x * self.p1.y - self.p1.x * self.p2.y)
            / (self.p2.x - self.p1.x)
    }
}

struct PowerLawFunction {
    p1: Point2d,
    p2: Point2d,
    b: f64,
}

impl PowerLawFunction {
    fn new(p1: Point2d, p2: Point2d, b: f64) -> Self {
        PowerLawFunction { p1, p2, b }
    }
}

impl ScalarFunction for PowerLawFunction {
    fn compute(&self, x: f64) -> f64 {
        assert!(self.p2.x != self.p1.x);
        ((self.p2.y - self.p1.y) * x.powf(self.b) + self.p2.x * self.p1.y - self.p1.x * self.p2.y)
            / (self.p2.x - self.p1.x)
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
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

    fn is_piece_overlapping(&self, piece: &Piece) -> bool {
        self.functions
            .iter()
            .any(|f| f.piece.contains(piece.closed_begin) || f.piece.contains(piece.open_end))
    }

    fn add_piece(mut self, piece: Piece, foo: Box<dyn ScalarFunction>) -> Result<Self, String> {
        if self.is_piece_overlapping(&piece) && self.functions.len() > 0 {
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
        Colormap { red, green, blue }
    }

    fn put_in_range(x: f64) -> f64 {
        // if x < 0.0 {
        //     return 0.0;
        // } else if x > 1.0 {
        //     return 1.0;
        // } else {
        //     return x;
        // }
        x
    }

    pub fn rgb_f(&self, x: f64) -> Color {
        assert!(x >= 0.0 && x <= 1.0);
        // println!("x = {}", x);
        Color::rgb_f(
            Colormap::put_in_range(self.red.compute(x)) as f32,
            Colormap::put_in_range(self.green.compute(x)) as f32,
            Colormap::put_in_range(self.blue.compute(x)) as f32,
        )
    }
}

pub fn earth() -> Colormap {
    let red = generate_earth_red().unwrap();
    let green = generate_earth_green().unwrap();
    let blue = generate_earth_blue().unwrap();
    Colormap::new(red, green, blue)
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
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.8),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.8),
                Point2d::new(p2, 0.9),
                0.9,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.9),
                Point2d::new(p3, 1.0),
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
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.5),
                Point2d::new(p2, 0.9),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.9),
                Point2d::new(p3, 1.0),
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
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.5),
                Point2d::new(p2, 0.7),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.7),
                Point2d::new(p3, 1.0),
                0.2,
            )),
        )
}

pub fn water() -> Colormap {
    let red = generate_water_red().unwrap();
    let green = generate_water_green().unwrap();
    let blue = generate_water_blue().unwrap();
    Colormap::new(red, green, blue)
}

fn generate_water_red() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.5),
                Point2d::new(p2, 0.7),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.7),
                Point2d::new(p3, 1.0),
                0.2,
            )),
        )
}

fn generate_water_green() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.5),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.5),
                Point2d::new(p2, 0.9),
                0.2,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.9),
                Point2d::new(p3, 1.0),
                0.2,
            )),
        )
}

fn generate_water_blue() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p1 = 3.0 / 8.0;
    let p2 = 6.0 / 8.0;
    let p3 = 1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(PowerLawFunction::new(
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.8),
                0.6,
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(PowerLawFunction::new(
                Point2d::new(p1, 0.8),
                Point2d::new(p2, 0.9),
                0.9,
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(PowerLawFunction::new(
                Point2d::new(p2, 0.9),
                Point2d::new(p3, 1.0),
                0.2,
            )),
        )
}

pub fn leeloo() -> Colormap {
    let red = generate_leeloo_red().unwrap();
    let green = generate_leeloo_green().unwrap();
    let blue = generate_leeloo_blue().unwrap();
    Colormap::new(red, green, blue)
}

fn generate_leeloo_red() -> Result<PiecewiseFunction, String> {
    let p0 = 0.0;
    let p2 = 3.0 / 8.0;
    let p3 = 5.0 / 8.0;
    let p4 = 7.0 / 8.0;
    let p5 = 1.0;
    let p6 = 9.0 / 8.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p2),
            Box::new(LinearFunction::new(
                Point2d::new(p0, 0.0),
                Point2d::new(p2, 0.0),
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(LinearFunction::new(
                Point2d::new(p2, 0.0),
                Point2d::new(p3, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p3, p4),
            Box::new(LinearFunction::new(
                Point2d::new(p3, 1.0),
                Point2d::new(p4, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p4, p5),
            Box::new(LinearFunction::new(
                Point2d::new(p4, 1.0),
                Point2d::new(p6, 0.0),
            )),
        )
}

fn generate_leeloo_green() -> Result<PiecewiseFunction, String> {
    let p0  =  0.0;
    let p1  =  1.0/8.0;
    let p2  =  3.0/8.0;
    let p3  =  5.0/8.0;
    let p4  =  7.0/8.0;
    let p5  =  1.0;
    let p6  =  9.0/8.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(LinearFunction::new(
                Point2d::new(p0, 0.0),
                Point2d::new(p1, 0.0),
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(LinearFunction::new(
                Point2d::new(p1, 0.0),
                Point2d::new(p2, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(LinearFunction::new(
                Point2d::new(p2, 1.0),
                Point2d::new(p3, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p3, p4),
            Box::new(LinearFunction::new(
                Point2d::new(p3, 1.0),
                Point2d::new(p4, 0.0),
            )),
        )?
        .add_piece(
            Piece::new(p4, p5),
            Box::new(LinearFunction::new(
                Point2d::new(p4, 0.0),
                Point2d::new(p6, 0.0),
            )),
        )
}

fn generate_leeloo_blue() -> Result<PiecewiseFunction, String> {
    let pm1 =  -1.0/8.0;
    let p0  =  0.0;
    let p1  =  1.0/8.0;
    let p2  =  3.0/8.0;
    let p3  =  5.0/8.0;
    let p5  =  1.0;

    PiecewiseFunction::new()
        .add_piece(
            Piece::new(p0, p1),
            Box::new(LinearFunction::new(
                Point2d::new(pm1, 0.0),
                Point2d::new(p1, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p1, p2),
            Box::new(LinearFunction::new(
                Point2d::new(p1, 1.0),
                Point2d::new(p2, 1.0),
            )),
        )?
        .add_piece(
            Piece::new(p2, p3),
            Box::new(LinearFunction::new(
                Point2d::new(p2, 1.0),
                Point2d::new(p3, 0.0),
            )),
        )?
        .add_piece(
            Piece::new(p3, p5),
            Box::new(LinearFunction::new(
                Point2d::new(p3, 0.0),
                Point2d::new(p5, 0.0),
            )),
        )
}
