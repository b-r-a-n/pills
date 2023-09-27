use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridColor {
    RED,
    BLUE,
    YELLOW,
    GREEN,
    ORANGE,
    PURPLE,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Orientation {
    Above,
    Right,
    Below,
    Left,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub enum Cell<T: Clone + Copy + PartialEq> {
    Empty,
    Virus(T, GridColor),
    Pill(T, GridColor, Option<Orientation>)
}

impl<T: Clone + Copy + PartialEq> Cell<T> {
    pub fn color(&self) -> Option<GridColor> {
        match self {
            Cell::Empty => None,
            Cell::Virus(_, color) => Some(*color),
            Cell::Pill(_, color, _) => Some(*color),
        }
    }

    pub fn get(&self) -> Option<T> {
        match self {
            Cell::Empty => None,
            Cell::Virus(t, _) => Some(*t),
            Cell::Pill(t, _, _) => Some(*t),
        }
    }

    fn is_pill(&self) -> bool {
        match self {
            Cell::Pill(_, _, _) => true,
            _ => false,
        }
    }

    fn is_virus(&self) -> bool {
        match self {
            Cell::Virus(_, _) => true,
            _ => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false,
        }
    }
}

#[derive(Clone, Default, Resource)]
pub struct Grid<T: Clone + Copy + PartialEq> {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Cell<T>>,
}

impl<T: Clone + Copy + PartialEq> Grid<T> {
    
    pub fn new(rows: usize, cols: usize) -> Self {
        Grid {
            rows,
            cols,
            cells: vec![Cell::Empty; rows * cols],
        }
    }

    pub fn resolve<F>(&self, cmp: F) -> Self where
        F: Fn(Cell<T>, Cell<T>) -> bool {
        let mut new_grid = self.clone();
        for row in 0..self.rows {
            let mut matches = 1;
            let mut pills = if self.get(row, 0).is_pill() { 1 } else { 0 };
            for col in 1..self.cols {
                if cmp(self.get(row, col), self.get(row, col - 1)) {
                    matches += 1;
                    if self.get(row, col).is_pill() { pills += 1; }
                } else {
                    if matches >= 4 && pills >= 1 {
                        for i in 0..matches {
                            new_grid.remove_piece(row, col - i - 1);
                        }
                    }
                    matches = 1;
                    pills = if self.get(row, col).is_pill() { 1 } else { 0 };
                }
            }
            if matches >= 4 && pills >= 1 {
                for i in 0..matches {
                    new_grid.remove_piece(row, self.cols - i - 1);
                }
            }
        }
        for col in 0..self.cols {
            let mut matches = 1;
            let mut pills = if self.get(0, col).is_pill() { 1 } else { 0 };
            for row in 1..self.rows {
                if cmp(self.get(row, col), self.get(row - 1, col)) {
                    matches += 1;
                    if self.get(row, col).is_pill() { pills += 1; }
                } else {
                    if matches >= 4 && pills >= 1 {
                        for i in 0..matches {
                            new_grid.remove_piece(row - i - 1, col);
                        }
                    }
                    matches = 1;
                    pills = if self.get(row, col).is_pill() { 1 } else { 0 };
                }
            }
            if matches >= 4 && pills >= 1 {
                for i in 0..matches {
                    new_grid.remove_piece(self.rows - i - 1, col);
                }
            }
        }
        new_grid
    }

    pub fn next(&self) -> Self {
        // Return a new grid that represents the next state
        let mut new_grid = Grid {
            rows: self.rows,
            cols: self.cols,
            cells: vec![Cell::Empty; self.rows * self.cols],
        };
        for col in 0..self.cols {
            new_grid.cells[col as usize] = self.cells[col as usize];
        }
        // Starting from the bottom row + 1, check if any cells can move down
        for row in 1..self.rows {
            for col in 0..self.cols {
                let cell = self.get(row, col);
                match cell {
                    Cell::Virus(_, _) => new_grid.cells[row * self.cols + col] = cell,
                    Cell::Empty => continue,
                    Cell::Pill(_, _, maybe_cell_orientation) => {
                        let below = new_grid.get(row - 1, col);
                        match below {
                            Cell::Empty => {
                                // Need to check the connected cell
                                match maybe_cell_orientation {
                                    Some(Orientation::Above) => new_grid.cells[(row-1) * self.cols + col] = cell,
                                    Some(Orientation::Below) => new_grid.cells[(row-1) * self.cols + col] = cell,
                                    Some(Orientation::Right) => {
                                        // If the cell to the right can fall, this one can as well
                                        if new_grid.get(row-1, col+1) == Cell::Empty {
                                            new_grid.cells[(row-1) * self.cols + col] = cell;
                                        } else {
                                            new_grid.cells[row * self.cols + col] = cell;
                                        }
                                    },
                                    Some(Orientation::Left) => {
                                        // We have already processed the cell to the left
                                        // If that cell is empty, then this cell can fall
                                        if new_grid.cells[row * self.cols + col - 1] == Cell::Empty {
                                            new_grid.cells[(row-1) * self.cols + col] = cell;
                                        } else {
                                            new_grid.cells[row * self.cols + col] = cell;
                                        }
                                    },
                                    None => new_grid.cells[(row-1) * self.cols + col] = cell,
                                }
                            },
                            _ => new_grid.cells[row * self.cols + col] = cell,
                        }
                    }
                }

            }
        }
        new_grid
    }

    pub fn get(&self, row: usize, col: usize) -> Cell<T> {
        // TODO panic if row or col is out of bounds
        return self.cells[row * self.cols + col];
    }

    pub fn get_paired(&self, row: usize, col: usize) -> (Cell<T>, Option<(Cell<T>, usize, usize)>) {
        // TODO panic if row or col is out of bounds
        let cell = self.get(row, col);
        match cell {
            Cell::Pill(_, _, maybe_orientation) => {
                match maybe_orientation {
                    Some(Orientation::Right) => {
                        return (cell, Some((self.get(row, col + 1), row, col + 1)));
                    },
                    Some(Orientation::Left) => {
                        return (cell, Some((self.get(row, col - 1), row, col - 1)));
                    },
                    Some(Orientation::Above) => {
                        return (cell, Some((self.get(row + 1, col), row + 1, col)));
                    },
                    Some(Orientation::Below) => {
                        return (cell, Some((self.get(row - 1, col), row - 1, col)));
                    },
                    _ => {
                        return (cell, None);
                    }
                }
            },
            _ => {
                return (cell, None);
            }
        }
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell<T>) {
        self.cells[row * self.cols + col] = cell;
    }

    fn remove_piece(&mut self, row: usize, col:usize) -> &mut Self {
        let (_, maybe_piece) = self.get_paired(row, col);
        self.set(row, col, Cell::Empty);
        if let Some((paired_cell, row, col)) = maybe_piece {
            match paired_cell {
                Cell::Pill(t, c, _) => {
                    self.set(row, col, Cell::Pill(t, c, None));
                },
                _ => {},
            }
        }
        self
    }

    fn remove_pill(&mut self, row: usize, col: usize) -> &mut Self {
        let (_, maybe_piece) = self.get_paired(row, col);
        self.set(row, col, Cell::Empty);
        if let Some((_, row, col)) = maybe_piece {
            self.set(row, col, Cell::Empty);
        }
        self
    }

    fn add_pill(&mut self, row: usize, col: usize, pill: (Cell<T>, Cell<T>)) -> bool {
        if row >= self.rows || col >= self.cols { return false; }
        let origin_cell = self.get(row, col);
        if !origin_cell.is_empty() { return false; }
        if origin_cell.is_pill() { return false; }
        match pill.0 {
            Cell::Pill(_, _, Some(orientation)) => {
                let (mut other_row, mut other_col) = (row, col);
                match orientation {
                    Orientation::Above => { 
                        if other_row >= self.rows - 1 { return false; }
                        other_row += 1; 
                    },
                    Orientation::Below => { 
                        if other_row <= 0 { return false; }
                        other_row -= 1; 
                    },
                    Orientation::Left => { 
                        if other_col <= 0 { return false; }
                        other_col -= 1; 
                    },
                    Orientation::Right => { 
                        if other_col >= self.cols - 1 { return false; }
                        other_col += 1; 
                    },
                }
                match self.get(other_row, other_col) {
                    Cell::Empty => {
                        self.set(row, col, pill.0);
                        self.set(other_row, other_col, pill.1);
                    },
                    _ => { return false; },
                }
            },
            _ => { return false; },
        }
        true
    }

    pub fn move_pill(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (pill, maybe_pill_info) = self.get_paired(from.0, from.1);
        self.remove_pill(from.0, from.1);
        if self.add_pill(to.0, to.1, (pill, maybe_pill_info.unwrap_or((Cell::Empty, 0, 0)).0)) {
            return true
        }
        self.add_pill(from.0, from.1, (pill, maybe_pill_info.unwrap_or((Cell::Empty, 0, 0)).0));
        false
    }

    // Left : Above -> Left -> Below -> Right -> Above
    // Right : Above -> Right -> Below -> Left -> Above
    pub fn rotate_pill(&mut self, at: (usize, usize), direction: Orientation) -> bool {
        let original_pair = self.get_paired(at.0, at.1);
        let new_pair: Option<(Cell<T>, Cell<T>)> = match original_pair {
            (Cell::Pill(t1, c1, Some(o1)), Some((Cell::Pill(t2, c2, Some(_)), _, _))) => {
                match (direction, o1) {
                    (Orientation::Left, Orientation::Above) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Left));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Right));
                        Some((p1, p2))
                    },
                    (Orientation::Left, Orientation::Left) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Below));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Above));
                        Some((p1, p2))
                    },
                    (Orientation::Left, Orientation::Below) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Right));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Left));
                        Some((p1, p2))
                    },
                    (Orientation::Left, Orientation::Right) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Above));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Below));
                        Some((p1, p2))
                    },
                    (Orientation::Right, Orientation::Above) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Right));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Left));
                        Some((p1, p2))
                    },
                    (Orientation::Right, Orientation::Right) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Below));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Above));
                        Some((p1, p2))
                    },
                    (Orientation::Right, Orientation::Below) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Left));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Right));
                        Some((p1, p2))
                    },
                    (Orientation::Right, Orientation::Left) => {
                        let p1 = Cell::Pill(t1, c1, Some(Orientation::Above));
                        let p2 = Cell::Pill(t2, c2, Some(Orientation::Below));
                        Some((p1, p2))
                    },
                    _ => { None }
                }
            },
            _ => { None },
        };
        if let Some((c1, c2)) = new_pair {
            self.remove_pill(at.0, at.1);
            if self.add_pill(at.0, at.1, (c1, c2)) {
                return true;
            }
            self.add_pill(at.0, at.1, (original_pair.0, original_pair.1.unwrap_or((Cell::Empty, 0, 0)).0));
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_grid() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Empty; 4],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty; 4]);
    }

    #[test]
    fn test_grid_with_floating_virus() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED)],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED)]);
    }

    #[test]
    fn test_grid_with_single_cell_piece_falling() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::RED, None)],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::RED, None), Cell::<u32>::Empty, Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_connected_cell_falling() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left)), Cell::<u32>::Empty, Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_connected_cell_that_cannot_fall_due_to_left_virus() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Virus(0, GridColor::YELLOW), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Virus(0, GridColor::YELLOW), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))]);
    }

    #[test]
    fn test_grid_with_connected_cell_that_cannot_fall_due_to_right_virus() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::YELLOW), Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::YELLOW), Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))]);
    }

    #[test]
    fn test_grid_with_connected_cell_that_cannot_fall_due_to_full_pill() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Left)), Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Left)), Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))])
    }

    #[test]
    fn test_grid_with_connected_cell_that_cannot_fall_due_to_left_pill() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Pill(0,GridColor::YELLOW, None), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Pill(0,GridColor::YELLOW, None), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Right)), Cell::<u32>::Pill(0,GridColor::RED, Some(Orientation::Left))])
    }

    #[test]
    fn test_grid_with_connected_cell_that_cannot_fall_because_of_itself() {
        let grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Above)), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Below)), Cell::<u32>::Empty],
        };
        let next_grid = grid.next();
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Pill(0,GridColor::YELLOW, Some(Orientation::Above)), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::BLUE, Some(Orientation::Below)), Cell::<u32>::Empty])

    }

    #[test]
    fn test_grid_with_vertical_match_resolves_to_empty() {
        let grid = Grid {
            rows: 4,
            cols: 2,
            cells: vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Pill(0,GridColor::RED, None), Cell::<u32>::Empty],
        };
        let next_grid = grid.resolve(|a, b| a.color().is_some() && a.color() == b.color());
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_horizontal_match_resolves_to_empty() {
        let grid = Grid {
            rows: 2,
            cols: 4,
            cells: vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Pill(0,GridColor::RED, None), Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty],
        };
        let next_grid = grid.resolve(|a, b| a.color().is_some() && a.color() == b.color());
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_vertical_match_no_pill_resolves_to_self() {
        let grid = Grid {
            rows: 4,
            cols: 2,
            cells: vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty],
        };
        let next_grid = grid.resolve(|a, b| a.color().is_some() && a.color() == b.color());
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_horizontal_match_resolves_to_self() {
        let grid = Grid {
            rows: 2,
            cols: 4,
            cells: vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty],
        };
        let next_grid = grid.resolve(|a, b| a.color().is_some() && a.color() == b.color());
        assert_eq!(next_grid.cells, vec![Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Virus(0, GridColor::RED), Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty, Cell::<u32>::Empty]);
    }

    #[test]
    fn test_grid_with_partial_pill_match_cleans_up_pill() {
        let grid = Grid {
            rows: 2,
            cols: 4,
            cells: vec![
                Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), 
                Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), 
                Cell::Pill(0, GridColor::RED, Some(Orientation::Above)), 
                Cell::Pill(0, GridColor::RED, Some(Orientation::Above)), 
                Cell::Empty, Cell::Empty, 
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), 
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below))]};
        let next_grid = grid.resolve(|a, b| a.color().is_some() && a.color() == b.color());
        assert_eq!(next_grid.cells, 
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, 
                Cell::Pill(0, GridColor::BLUE, None), 
                Cell::Pill(0, GridColor::BLUE, None)]);
    }

    #[test]
    fn test_moving_pill_in_empty_grid() {
        let mut grid = Grid {
            rows: 2,
            cols: 3,
            cells: vec![Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty],
        };
        assert!(grid.move_pill((0, 0), (0, 1)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty]);
        assert!(grid.move_pill((0, 1), (1, 1)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left))]);
        assert!(grid.move_pill((1, 1), (1, 0)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty]);
        assert!(grid.move_pill((1, 0), (0, 0)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty]);
    }

    #[test]
    fn test_moving_pill_at_left_edge_of_grid() {
        let mut grid = Grid {
            rows: 2,
            cols: 4,
            cells: vec![Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty],
        };
        assert!(!grid.move_pill((0, 1), (0, 0)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty]);
    }

    #[test]
    fn test_moving_pill_at_right_edge_of_grid() {
        let mut grid = Grid {
            rows: 2,
            cols: 4,
            cells: vec![Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty],
        };
        assert!(!grid.move_pill((0, 2), (0, 3)));
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left)), Cell::Empty, Cell::Empty, Cell::Empty, Cell::Empty]);
    }

    #[test]
    fn test_removing_piece_updates_left_connected_piece() {
        let mut grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left))],
        };
        grid.remove_piece(1, 0);
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, None)]);
    }

    #[test]
    fn test_removing_piece_updates_right_connected_piece() {
        let mut grid = Grid {
            rows: 2,
            cols: 2,
            cells: vec![Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, Some(Orientation::Right)), Cell::Pill(0, GridColor::RED, Some(Orientation::Left))],
        };
        grid.remove_piece(1, 1);
        assert_eq!(
            grid.cells, 
            vec![Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::RED, None), Cell::Empty]);
    }

    #[test]
    fn test_rotating_pill_left_on_empty_grid() {
        let mut grid = Grid {
            rows: 3,
            cols: 3,
            cells: vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty]};
        assert!(grid.rotate_pill((1, 1), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)), Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Empty]);

        assert!(grid.rotate_pill((1, 1), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Empty]);
        assert!(grid.rotate_pill((1, 1), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)),
                Cell::Empty, Cell::Empty, Cell::Empty]);
        assert!(grid.rotate_pill((1, 1), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty]);
    }

    #[test]
    fn test_rotating_pill_right_on_empty_grid() {
        let mut grid = Grid {
            rows: 3,
            cols: 3,
            cells: vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty]};
        assert!(grid.rotate_pill((1, 1), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)),
                Cell::Empty, Cell::Empty, Cell::Empty]);

        assert!(grid.rotate_pill((1, 1), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Empty]);
        assert!(grid.rotate_pill((1, 1), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)), Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Empty]);
        assert!(grid.rotate_pill((1, 1), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty]);
    }

    #[test]
    fn test_rotating_pill_right_at_right_boundary() {
        let mut grid = Grid {
            rows: 3,
            cols: 3,
            cells: vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)),
                Cell::Empty, Cell::Empty, Cell::Empty]};
        assert!(grid.rotate_pill((1, 2), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)),
                Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below))]);
        assert!(!grid.rotate_pill((1, 2), Orientation::Right));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)),
                Cell::Empty, Cell::Empty, Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below))]);
    }

    #[test]
    fn test_rotating_pill_left_at_left_boundary() {
        let mut grid = Grid {
            rows: 3,
            cols: 3,
            cells: vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Right)), Cell::Pill(0, GridColor::BLUE, Some(Orientation::Left)), Cell::Empty,
                Cell::Empty, Cell::Empty, Cell::Empty]};
        assert!(grid.rotate_pill((1, 0), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty, Cell::Empty, 
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty, Cell::Empty]);
        assert!(!grid.rotate_pill((1, 0), Orientation::Left));
        assert_eq!(
            grid.cells,
            vec![
                Cell::Empty, Cell::Empty, Cell::Empty,
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Above)), Cell::Empty, Cell::Empty, 
                Cell::Pill(0, GridColor::BLUE, Some(Orientation::Below)), Cell::Empty, Cell::Empty]);
    }
}