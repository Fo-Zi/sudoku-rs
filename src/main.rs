use std::collections::HashMap;

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

    //
    fn make_move(&self, sudoku_move: SudokuMove) -> SudokuMoveResult {
        let mut invalid_cells_coordinates = Vec::new();

        // Adds all duplicate cells in the sub-grid where the move was attempted ->
        if let Some(subgrid_entry) = self.sub_grids.get(&sudoku_move.cell_coordinate.sub_grid) {
            if let Some(duplicates) = subgrid_entry.get_duplicates() {
                for duplicate_cell_pos in duplicates.iter() {
                    invalid_cells_coordinates.push(CellCoordinate {
                        sub_grid: sudoku_move.cell_coordinate.sub_grid,
                        cell: *duplicate_cell_pos,
                    });
                }
            }
        }

        SudokuMoveResult::Ok
    }
}

/// Main app managing the Sudoku UI
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
