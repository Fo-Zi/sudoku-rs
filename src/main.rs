use std::{cell::Cell, collections::HashMap};

use eframe::egui;

fn keys_with_duplicate_values<K: Eq + std::hash::Hash + Clone, V: Eq + std::hash::Hash>(
    map: &HashMap<K, V>,
) -> Vec<K> {
    let mut value_counts: HashMap<&V, usize> = HashMap::new();

    // Count occurrences of each value
    for value in map.values() {
        *value_counts.entry(value).or_insert(0) += 1;
    }

    // Collect keys where the value appears more than once
    map.iter()
        .filter(|(_, value)| value_counts[value] > 1)
        .map(|(key, _)| key.clone())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Row {
    Upper = 1,
    Center = 2,
    Bottom = 3,
}

impl Row {
    fn all() -> &'static [Row] {
        &[Row::Upper, Row::Center, Row::Bottom]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Column {
    Left = 1,
    Center = 2,
    Right = 3,
}

impl Column {
    fn all() -> &'static [Column] {
        &[Column::Left, Column::Center, Column::Right]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PositionId {
    row: Row,
    column: Column,
}

/// Represents a 3x3 section of the Sudoku board
#[derive(Default, Clone)]
struct SubGrid {
    cells: HashMap<PositionId, Option<u8>>,
}

struct SubGridMove {
    cell: PositionId,
    value: u8,
}

enum SubgridMoveResult {
    Ok,
    Invalid(Vec<PositionId>),
}

impl SubGrid {
    /// Creates a new empty 3x3 subgrid
    fn new() -> Self {
        let mut empty_cells: HashMap<PositionId, Option<u8>> = HashMap::new();
        for row in Row::all() {
            for column in Column::all() {
                let position_id = PositionId {
                    row: *row,
                    column: *column,
                };
                empty_cells.insert(position_id, None);
            }
        }

        Self { cells: empty_cells }
    }

    fn update_value(&mut self, key: PositionId, value: u8) {
        if let Some(entry) = self.cells.get_mut(&key) {
            *entry = Some(value); // Only updates existing keys
        }
    }

    fn get_value(&self, key: PositionId) -> Option<u8> {
        self.cells.get(&key).copied()?
    }

    // Returns vec with all the positions where there is a duplicate
    fn get_duplicates(&self) -> Option<Vec<PositionId>> {
        let duplicates = keys_with_duplicate_values(&self.cells);
        if duplicates.len() > 0 {
            Some(duplicates)
        } else {
            None
        }
    }

    pub fn make_move(&self, sub_grid_move: SubGridMove) -> SubgridMoveResult {
        let mut invalid_cells = Vec::new();

        if let Some(_entry) = self.cells.get(&sub_grid_move.cell) {
            if let Some(duplicates) = self.get_duplicates() {
                invalid_cells.extend(duplicates);
            };
        }

        if invalid_cells.len() > 0 {
            SubgridMoveResult::Invalid(invalid_cells)
        } else {
            SubgridMoveResult::Ok
        }
    }
}

struct CellCoordinate {
    sub_grid: PositionId,
    cell: PositionId,
}

/// Represents the full 9x9 Sudoku board
#[derive(Default)]
struct SudokuBoard {
    sub_grids: HashMap<PositionId, SubGrid>,
}

struct SudokuMove {
    cell_coordinate: CellCoordinate,
    value: u8,
}

enum SudokuMoveResult {
    Ok,
    Invalid(Vec<CellCoordinate>),
}

impl SudokuBoard {
    /// Creates an empty Sudoku board
    fn new() -> Self {
        let mut sub_grids: HashMap<PositionId, SubGrid> = HashMap::new();
        for row in Row::all() {
            for col in Column::all() {
                sub_grids.insert(
                    PositionId {
                        row: *row,
                        column: *col,
                    },
                    SubGrid::new(),
                );
            }
        }

        Self {
            sub_grids: sub_grids,
        }
    }

    fn update_value(&mut self, cell_coordinate: CellCoordinate, value: u8) {
        if let Some(subgrid_entry) = self.sub_grids.get_mut(&cell_coordinate.sub_grid) {
            subgrid_entry.update_value(cell_coordinate.cell, value);
        }
    }

    fn get_value(&self, cell_coordinate: CellCoordinate) -> Option<u8> {
        let sub_grid = self.sub_grids.get(&cell_coordinate.sub_grid)?;
        sub_grid.get_value(cell_coordinate.cell)
    }

    fn get_row_duplicates(&self, sudoku_move: &SudokuMove) -> Option<Vec<CellCoordinate>> {
        let mut row_duplicates = Vec::new();
        for sub_grid_col in Column::all() {
            let sub_grid_pos = PositionId {
                row: sudoku_move.cell_coordinate.sub_grid.row,
                column: *sub_grid_col,
            };
            if let Some(subgrid_entry) = self.sub_grids.get(&sub_grid_pos) {
                for cell_col in Column::all() {
                    let cell_pos = PositionId {
                        row: sudoku_move.cell_coordinate.cell.row,
                        column: *cell_col,
                    };
                    if let Some(value) = subgrid_entry.get_value(cell_pos) {
                        if value == sudoku_move.value {
                            row_duplicates.push(CellCoordinate {
                                sub_grid: sub_grid_pos,
                                cell: cell_pos,
                            });
                        }
                    }
                }
            }
        }

        if row_duplicates.len() > 0 {
            Some(row_duplicates)
        } else {
            None
        }
    }

    fn get_column_duplicates(&self, sudoku_move: &SudokuMove) -> Option<Vec<CellCoordinate>> {
        let mut col_duplicates = Vec::new();
        for sub_grid_row in Row::all() {
            let sub_grid_pos = PositionId {
                row: *sub_grid_row,
                column: sudoku_move.cell_coordinate.sub_grid.column,
            };
            if let Some(subgrid_entry) = self.sub_grids.get(&sub_grid_pos) {
                for cell_row in Row::all() {
                    let cell_pos = PositionId {
                        row: *cell_row,
                        column: sudoku_move.cell_coordinate.cell.column,
                    };
                    if let Some(value) = subgrid_entry.get_value(cell_pos) {
                        if value == sudoku_move.value {
                            col_duplicates.push(CellCoordinate {
                                sub_grid: sub_grid_pos,
                                cell: cell_pos,
                            });
                        }
                    }
                }
            }
        }

        if col_duplicates.len() > 0 {
            Some(col_duplicates)
        } else {
            None
        }
    }

    //
    fn make_move(&self, sudoku_move: &SudokuMove) -> SudokuMoveResult {
        let mut invalid_cells_coordinates = Vec::new();

        // Adds all duplicate cells in the sub-grid where the move was attempted ->
        if let Some(subgrid_entry) = self.sub_grids.get(&sudoku_move.cell_coordinate.sub_grid) {
            if let SubgridMoveResult::Invalid(invalid_cells) =
                subgrid_entry.make_move(SubGridMove {
                    cell: sudoku_move.cell_coordinate.cell,
                    value: sudoku_move.value,
                })
            {
                for invalid_cell in invalid_cells.iter() {
                    invalid_cells_coordinates.push(CellCoordinate {
                        sub_grid: sudoku_move.cell_coordinate.sub_grid,
                        cell: *invalid_cell,
                    });
                }
            };
        }

        if let Some(row_duplicates) = self.get_row_duplicates(sudoku_move) {
            invalid_cells_coordinates.extend(row_duplicates);
        }

        if let Some(col_duplicates) = self.get_column_duplicates(sudoku_move) {
            invalid_cells_coordinates.extend(col_duplicates);
        }

        if invalid_cells_coordinates.len() > 0 {
            SudokuMoveResult::Invalid(invalid_cells_coordinates)
        } else {
            SudokuMoveResult::Ok
        }
    }
}

struct SudokuApp {
    board: SudokuBoard,
    move_history: Vec<SudokuMove>,
    nr_mistakes: u8,
}

impl SudokuApp {
    fn new() -> Self {
        Self {
            board: SudokuBoard::new(),
            move_history: Vec::new(),
            nr_mistakes: 0_u8,
        }
    }

    fn update_grid(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sudoku Board");
        });
    }
}

impl eframe::App for SudokuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_grid(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sudoku Grid",
        options,
        Box::new(|_cc| Ok(Box::new(SudokuApp::new()))),
    )
}
 
#[cfg(test)]
mod tests {
    use super::*; // Import functions from the parent module

    #[test]
    fn keys_with_duplicate_values_empty_input() {
        let mut empty_test_cells: HashMap<PositionId, Option<u8>> = HashMap::new();
        let vec = keys_with_duplicate_values(&empty_test_cells);
        assert_eq!(vec.len() , 0);
    }
    
    #[test]
    fn keys_with_duplicate_values_no_duplicates_input() {
        let mut no_duplicate_test_cells: HashMap<PositionId, Option<u8>> = HashMap::new();
        no_duplicate_test_cells.insert(
            PositionId { 
                row: Row::Upper, 
                column: Column::Left
            }, 
            Some(1)
        );
        no_duplicate_test_cells.insert(
            PositionId { 
                row: Row::Upper, 
                column: Column::Center
            }, 
            Some(2)
        );
        no_duplicate_test_cells.insert(
            PositionId { 
                row: Row::Upper, 
                column: Column::Right
            }, 
            Some(3)
        );
        let vec = keys_with_duplicate_values(&no_duplicate_test_cells);
        assert_eq!(vec.len() , 0);
    }
    
    #[test]
    fn keys_with_duplicate_values_detects_duplicates() {
        let mut no_duplicate_test_cells: HashMap<PositionId, Option<u8>> = HashMap::new();

        let position_match_1 = PositionId { 
            row: Row::Upper, 
            column: Column::Left
        };

        let position_match_2 = PositionId { 
            row: Row::Upper, 
            column: Column::Center
        }; 
        no_duplicate_test_cells.insert(
            position_match_1
            , 
            Some(1)
        );
        no_duplicate_test_cells.insert(
        position_match_2,
            Some(1)
        );
        no_duplicate_test_cells.insert(
            PositionId { 
                row: Row::Upper, 
                column: Column::Right
            }, 
            Some(3)
        );
        no_duplicate_test_cells.insert(
            PositionId { 
                row: Row::Upper, 
                column: Column::Right
            }, 
            Some(4)
        );
        let matched_keys = keys_with_duplicate_values(&no_duplicate_test_cells);
        assert_eq!(matched_keys.len() , 2);
        let mut was_match_1_returned = false;
        let mut was_match_2_returned = false;
        for matched_key in matched_keys {
            if matched_key == position_match_1 {
                was_match_1_returned = true;
            }
            if matched_key == position_match_2 {
                was_match_2_returned = true;
            } 
        }
        assert_eq!(true, was_match_1_returned & was_match_2_returned);

    }

}