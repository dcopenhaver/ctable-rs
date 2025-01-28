#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    truncate_at: usize,
    justification: Justification,
    max_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justification {
    Left,
    Right,
}

impl Column {
    
    /// Creates a new Column with the given name, truncation width, and justification.
    /// If truncate_at is 0, no truncation will occur.
    /// The effective truncation width will be the maximum of the provided width
    /// and the length of the column name to ensure headers are never truncated.
    pub fn new(name: impl Into<String>, truncate_at: usize, justification: Justification) -> Self {
        let name = name.into();
        let name_len = name.chars().count();
        
        // If truncation is requested (> 0), ensure minimum width of 3 for "..."
        let effective_truncate = if truncate_at > 0 {
            truncate_at.max(3).max(name_len)
        } else {
            truncate_at
        };
        
        Column {
            name,
            truncate_at: effective_truncate,
            justification,
            max_length: name_len,
        }
    }

    /// Sets the justification (Left or Right) for this column
    pub fn set_justification(&mut self, j: Justification) {
        self.justification = j;
    }

    /// Updates the maximum length of the column based on the content.
    /// For multiline values, considers the longest line.
    fn update_max_length(&mut self, value: &str) {
        for line in value.split('\n') {
            let len = line.chars().count();
            if len > self.max_length {
                self.max_length = len;
            }
        }
    }

    /// Formats a single cell's content for display in the table.
    /// Handles:
    /// - Splitting multiline content into separate lines (split by \n)
    /// - Truncating lines that exceed max width (adding "...")
    /// - Padding lines to match column width
    /// - Applying left/right justification
    /// Returns a vector of formatted strings, one for each line in the cell.
    fn format_cell(&self, cell_value: &str) -> Vec<String> {
        
        // Split the value into lines (if newlines (\n) are present in cell_value this is how multiline values are handled)
        let lines_of_cell: Vec<&str> = cell_value.split('\n').collect();
        
        lines_of_cell.into_iter()
            .map(|line| {
                let value_len = line.chars().count();
                let mut result = if self.truncate_at > 0 && value_len > self.truncate_at {
                    // Truncate the string if needed, leaving room for "..."
                    let truncate_pos = self.truncate_at.saturating_sub(3);
                    let mut truncated = line.chars().take(truncate_pos).collect::<String>();
                    truncated.push_str("...");
                    truncated
                } else {
                    line.to_string()
                };

                // Pad the string based on justification
                let width = if self.truncate_at > 0 {
                    self.truncate_at
                } else {
                    self.max_length
                };

                if result.chars().count() < width {
                    let padding = " ".repeat(width - result.chars().count());
                    match self.justification {
                        Justification::Left => result.push_str(&padding),
                        Justification::Right => result = format!("{}{}", padding, result),
                    }
                }

                result
            })
            .collect()
    }

    /// Creates an empty string of spaces matching the column's width.
    /// Used for padding multiline rows where some columns have fewer lines than others.
    fn format_empty(&self) -> String {
        " ".repeat(if self.truncate_at > 0 {
            self.truncate_at
        } else {
            self.max_length
        })
    }
}

#[derive(Debug)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
}

impl Table {
    
    /// Creates a new Table with the specified columns
    pub fn new(columns: Vec<Column>) -> Self {
        Table {
            columns,
            rows: Vec::new(),
        }
    }

    /// Adds a row to the table. Returns an error if the number of values
    /// doesn't match the number of columns.
    pub fn add_row(&mut self, row: Vec<String>) -> Result<(), String> {
        if row.len() != self.columns.len() {
            return Err(format!(
                "Row has {} columns, expected {}",
                row.len(),
                self.columns.len()
            ));
        }

        // Update max lengths for each column
        for (col, value) in self.columns.iter_mut().zip(row.iter()) {
            col.update_max_length(value);
        }

        self.rows.push(row);
        Ok(())
    }
}

/// Implements the Display trait to enable formatting the table as a string.
/// Handles multiline content, column alignment, and proper spacing.
impl std::fmt::Display for Table {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.columns.is_empty() {
            return Ok(());
        }

        // Format header
        let header: Vec<String> = self.columns
            .iter()
            .map(|col| col.format_cell(&col.name)[0].clone())
            .collect();
        
        writeln!(f, "{}", header.join(" "))?;

        // Format separator
        let separator: Vec<String> = self.columns
            .iter()
            .map(|col| {
                let width = if col.truncate_at > 0 {
                    col.truncate_at
                } else {
                    col.max_length
                };
                "-".repeat(width)
            })
            .collect();
        writeln!(f, "{}", separator.join(" "))?;

        // Format rows with multiline support
        for row in &self.rows {
            // Convert each cell into a vector of formatted lines
            let formatted_cells: Vec<Vec<String>> = self.columns
                .iter()
                .zip(row)
                .map(|(col, value)| col.format_cell(value))
                .collect();

            // Find the maximum number of lines in any cell of this row
            let max_lines = formatted_cells
                .iter()
                .map(|cell| cell.len())
                .max()
                .unwrap_or(1);

            // Print each line of the row
            for line_idx in 0..max_lines {
                let line: Vec<String> = formatted_cells
                    .iter()
                    .zip(self.columns.iter())
                    .map(|(cell, col)| {
                        if line_idx < cell.len() {
                            cell[line_idx].clone()
                        } else {
                            col.format_empty()
                        }
                    })
                    .collect();
                writeln!(f, "{}", line.join(" "))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_table() {
        let mut table = Table::new(vec![
            Column::new("Name", 0, Justification::Left),
            Column::new("Age", 0, Justification::Left),
            Column::new("City", 0, Justification::Left),
        ]);

        table.add_row(vec![
            "John Doe".to_string(),
            "30".to_string(),
            "New York".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "Jane Smith".to_string(),
            "25".to_string(),
            "Los Angeles".to_string(),
        ]).unwrap();

        println!("\n=== Basic Table Test ===\n\n{}\n", table);
    }

    #[test]
    fn test_truncation() {
        let mut table = Table::new(vec![
            Column::new("Name", 10, Justification::Left),
            Column::new("Description", 35, Justification::Left),
        ]);

        table.add_row(vec![
            "John Doe".to_string(),
            "A very long description that should be truncated".to_string(),
        ]).unwrap();

        println!("\n=== Truncation Test ===\n\n{}\n", table);
    }

    #[test]
    fn test_justification() {
        let cols = vec![
            Column::new("ID-R", 0, Justification::Right),
            Column::new("Name-L", 0, Justification::Left),
            Column::new("Balance-R", 0, Justification::Right),
        ];

        let mut table = Table::new(cols);

        table.add_row(vec![
            "1".to_string(),
            "John Doe".to_string(),
            "100.00".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "2".to_string(),
            "Jane Smith".to_string(),
            "250.50".to_string(),
        ]).unwrap();

        println!("\n=== Justification Test ===\n\n{}\n", table);
    }

    #[test]
    fn test_multiline() {
        let mut table = Table::new(vec![
            Column::new("Name", 0, Justification::Left),
            Column::new("Description", 0, Justification::Left),
            Column::new("Status", 0, Justification::Left),
        ]);

        table.add_row(vec![
            "John Doe".to_string(),
            "Software Engineer\nSpecializes in:\n- Rust\n- Go".to_string(),
            "Active".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "Jane Smith".to_string(),
            "Project Manager".to_string(),
            "On Leave\nReturns next week".to_string(),
        ]).unwrap();

        println!("\n=== Multiline Test ===\n\n{}\n", table);
    }

    #[test]
    fn test_all() {
        let mut table = Table::new(vec![
            Column::new("Name", 0, Justification::Left),
            Column::new("Description", 35, Justification::Left),
            Column::new("Status", 0, Justification::Left),
            Column::new("Value Right1", 5, Justification::Right),
            Column::new("Value Right2", 0, Justification::Right),
        ]);

        table.add_row(vec![
            "John Doe".to_string(),
            "Software Engineer with the following data data data data data\nSpecializes in:\n- Rust\n- Go".to_string(),
            "Active".to_string(),
            "1000.00".to_string(),
            "2000.00".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "Jane Smith".to_string(),
            "Project Manager".to_string(),
            "On Leave\nReturns next week".to_string(),
            "3000.00".to_string(),
            "4000.00".to_string(),
        ]).unwrap();

        println!("\n=== All Features Test ===\n\n{}\n", table);
    }
}
