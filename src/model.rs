use std::slice;
use std::ops::{Index, IndexMut};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Color {
    Blue,
    Green,
    Red,
    Yellow
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Ring {
    Top,
    Middle,
    Bottom
}

impl Ring {
    pub fn radius(&self, width: f64, height: f64) -> f64 {
        let dim = width.min(height) / 2.;
        let factor = match self {
            &Ring::Top => 0.7,
            &Ring::Middle => 0.4,
            &Ring::Bottom => 0.1
        };
        dim - (dim * factor)
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Layer([Option<Color>; 9]);

pub type Line = (Option<Color>, Option<Color>, Option<Color>);

fn matching_color(line: Line) -> Option<Color> {
    if let (Some(c1), Some(c2), Some(c3)) = line {
        if c1 == c2 && c2 == c3 {
            return Some(c1);
        }
    }
    None
}

impl Default for Layer {
    fn default() -> Layer {
        Layer::empty()
    }
}

impl Layer {
   pub fn new(arr: [Option<Color>; 9]) -> Layer {
        Layer(arr)
    }

    pub fn empty() -> Layer {
        Layer([None; 9])
    }

    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Option<Color>> {
        self.0.iter()
    }

    fn three_in_row(&self) -> Option<Color> {
        let rows = (0..3).flat_map(|n| matching_color(self.row(n)));
        let cols = (0..3).flat_map(|n| matching_color(self.column(n)));
        let desc = matching_color(self.descending()).into_iter();
        let asc = matching_color(self.ascending()).into_iter();
        rows.chain(cols).chain(desc).chain(asc).next()
    }

    pub fn full(&self) -> bool {
        self.0.iter().all(|p| p.is_some())
    }
}

pub fn location_to_index(width: usize, (x, y): Location) -> usize {
    y * width + x
}

pub fn index_to_location(width: usize, i: usize) -> Location {
    let row = i / width;
    let column = i - (row * width);
    (column, row)
}

pub type Location = (usize, usize);

impl Index<Location> for Layer {
    type Output = Option<Color>;

    fn index(&self, ind: Location) -> &Option<Color> {
        &self.0[location_to_index(3, ind)]
    }
}

impl IndexMut<Location> for Layer {
    fn index_mut(&mut self, ind: Location) -> &mut Option<Color> {
        &mut self.0[location_to_index(3, ind)]
    }
}

impl Lines for Layer {
    fn row(&self, ind: usize) -> Line  {
        (self[(0, ind)], self[(1, ind)], self[(2, ind)])
    }

    fn column(&self, ind: usize) -> Line {
        (self[(ind, 0)], self[(ind, 1)], self[(ind, 2)])
    }

    fn descending(&self) -> Line {
        (self[(0,0)], self[(1,1)], self[(2,2)])
    }

    fn ascending(&self) -> Line {
        (self[(0,2)], self[(1,1)], self[(2,0)])
    }
}

#[derive(Debug,PartialEq,Eq)]
pub struct Board {
    top: Layer,
    middle: Layer,
    bottom: Layer
}

impl Default for Board {
    fn default() -> Board {
        Board::empty()
    }
}

impl Board {
    pub fn new(top: Layer, middle: Layer, bottom: Layer) -> Board {
        Board {
            top: top,
            middle: middle,
            bottom: bottom
        }
    }

    pub fn empty() -> Board {
        Board {
            top: Layer::empty(),
            middle: Layer::empty(),
            bottom: Layer::empty()
        }
    }

    pub fn winner(&self) -> Option<Color> {
        self.full_stack()
            .or(self.three_of_same())
            .or(self.three_in_order())
    }

    fn full_stack(&self) -> Option<Color> {
        for stack in izip!(self.top.iter(),
                           self.middle.iter(),
                           self.bottom.iter()) {
            if let (&Some(c1), &Some(c2), &Some(c3)) = stack {
                if c1 == c2 && c2 == c3 {
                    return Some(c1)
                }
            }
        }
        None
    }

    fn three_of_same(&self) -> Option<Color> {
        self.top.three_in_row()
            .or(self.middle.three_in_row())
            .or(self.bottom.three_in_row())
    }

    fn three_in_order(&self) -> Option<Color> {
        fn helper<T>(lines: T) -> Option<Color>
            where T: Lines {
            let rows = (0..3).flat_map(|n| matching_color(lines.row(n)));
            let cols = (0..3).flat_map(|n| matching_color(lines.column(n)));
            let desc = matching_color(lines.descending()).into_iter();
            let asc = matching_color(lines.ascending()).into_iter();
            rows.chain(cols).chain(desc).chain(asc).next()
        }
        helper(self.downward_view()).or(helper(self.upward_view()))
    }

    pub fn downward_view(&self) -> DownwardView {
        DownwardView(self)
    }

    pub fn upward_view(&self) -> UpwardView {
        UpwardView(self)
    }

    pub fn full(&self) -> bool {
        self.top.full() && self.middle.full() && self.bottom.full()
    }

    pub fn get_ring(&self, ind: Location, ring: Ring) -> Option<Color> {
        let layer = match ring {
            Ring::Top => &self.top,
            Ring::Middle => &self.middle,
            Ring::Bottom => &self.bottom
        };
        layer[ind]
    }

    pub fn set_ring(&mut self, ind: Location, ring: Ring, color: Color) {
        let layer = match ring {
            Ring::Top => &mut self.top,
            Ring::Middle => &mut self.middle,
            Ring::Bottom => &mut self.bottom
        };
        layer[ind] = Some(color);
    }

}

pub trait Lines {
    fn row(&self, usize) -> Line;
    fn column(&self, usize) -> Line;
    fn ascending(&self) -> Line;
    fn descending(&self) -> Line;
}

pub struct DownwardView<'a>(&'a Board);
pub struct UpwardView<'a>(&'a Board);

pub trait LineAccessor {
    fn get_line_with<F>(&self, F) -> Line
        where F: Fn(&Layer) -> Line;
}

impl<'a> LineAccessor for UpwardView<'a> {
    fn get_line_with<F>(&self, accessor: F) -> Line
        where F: Fn(&Layer) -> Line {
        stacked_layers(&self.0.bottom, &self.0.middle, &self.0.top, accessor)
    }
}

impl<'a> LineAccessor for DownwardView<'a> {
    fn get_line_with<F>(&self, accessor: F) -> Line
        where F: Fn(&Layer) -> Line {
        stacked_layers(&self.0.top, &self.0.middle, &self.0.bottom, accessor)
    }
}

impl<T> Lines for T where T: LineAccessor {
    fn row(&self, ind: usize) -> Line {
        self.get_line_with(|l| l.row(ind))
    }

    fn column(&self, ind: usize) -> Line {
        self.get_line_with(|l| l.column(ind))
    }

    fn ascending(&self) -> Line {
        self.get_line_with(|l| l.ascending())
    }

    fn descending(&self) -> Line {
        self.get_line_with(|l| l.descending())
    }
}

fn stacked_layers<F>(a: &Layer, b: &Layer, c:&Layer, accessor: F) -> Line
    where F: Fn(&Layer) -> Line {
    let (a, _, _) = accessor(a);
    let (_, b, _) = accessor(b);
    let (_, _, c) = accessor(c);
    (a, b, c)
}

// Tests

#[test]
fn test_layer_row() {
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Green), Some(Color::Blue),
        Some(Color::Blue), Some(Color::Red), Some(Color::Blue),
        Some(Color::Yellow), Some(Color::Blue), Some(Color::Yellow)
    ]);
    assert_eq!(layer.row(0), (Some(Color::Yellow), Some(Color::Green), Some(Color::Blue)));
    assert_eq!(layer.row(1), (Some(Color::Blue), Some(Color::Red), Some(Color::Blue)));
    assert_eq!(layer.row(2), (Some(Color::Yellow), Some(Color::Blue), Some(Color::Yellow)));
}

#[test]
fn test_layer_column() {
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Green), Some(Color::Blue),
        Some(Color::Blue), Some(Color::Red), Some(Color::Blue),
        Some(Color::Yellow), Some(Color::Blue), Some(Color::Yellow)
    ]);
    assert_eq!(layer.column(0), (Some(Color::Yellow), Some(Color::Blue), Some(Color::Yellow)));
    assert_eq!(layer.column(1), (Some(Color::Green), Some(Color::Red), Some(Color::Blue)));
    assert_eq!(layer.column(2), (Some(Color::Blue), Some(Color::Blue), Some(Color::Yellow)));
}

#[test]
fn test_layer_three_in_row() {
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Yellow), Some(Color::Yellow),
        Some(Color::Blue), Some(Color::Red), Some(Color::Blue),
        Some(Color::Green), Some(Color::Blue), Some(Color::Red)
    ]);
    assert_eq!(layer.three_in_row(), Some(Color::Yellow));
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Blue), Some(Color::Red),
        Some(Color::Blue), Some(Color::Red), Some(Color::Blue),
        Some(Color::Red), Some(Color::Blue), Some(Color::Green)
    ]);
    assert_eq!(layer.three_in_row(), Some(Color::Red));
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Blue), Some(Color::Green),
        Some(Color::Blue), Some(Color::Red), Some(Color::Green),
        Some(Color::Red), Some(Color::Blue), Some(Color::Green)
    ]);
    assert_eq!(layer.three_in_row(), Some(Color::Green));
    let layer = Layer::new([
        Some(Color::Yellow), Some(Color::Blue), Some(Color::Green),
        Some(Color::Blue), Some(Color::Red), Some(Color::Yellow),
        Some(Color::Red), Some(Color::Blue), Some(Color::Green)
    ]);
    assert_eq!(layer.three_in_row(), None);
}

#[test]
fn test_board_full_stack() {
    let layer = Layer::new([
        None, None, None,
        None, Some(Color::Red), None,
        None, None, None
    ]);
    let board = Board::new(layer.clone(), layer.clone(), layer);
    assert_eq!(board.full_stack(), Some(Color::Red));
}

#[test]
fn test_board_three_in_order() {
    let top = Layer::new([
        Some(Color::Blue), None, None,
        None, None, None,
        None, None, None
    ]);
    let middle = Layer::new([
        None, None, None,
        None, Some(Color::Blue), None,
        None, None, None
    ]);
    let bottom = Layer::new([
        None, None, None,
        None, None, None,
        None, None, Some(Color::Blue)
    ]);
    let board = Board::new(top.clone(), middle.clone(), bottom.clone());
    assert_eq!(board.three_in_order(), Some(Color::Blue));
    let board = Board::new(bottom, middle, top);
    assert_eq!(board.three_in_order(), Some(Color::Blue));
}
