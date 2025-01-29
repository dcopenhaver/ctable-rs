const MAX_TRUNCATE_WIDTH: usize = 5000;
const MAX_TABLE_ROWS: usize = 5_000_000;
const MAX_CELL_LINES: usize = 5000;

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
    /// 
    /// # Errors
    /// - If name is empty
    /// - If truncate_at exceeds MAX_TRUNCATE_WIDTH (5000)
    pub fn new(name: impl Into<String>, truncate_at: usize, justification: Justification) -> Result<Self, String> {
        let name = name.into();
        if name.is_empty() {
            return Err("Column::new: column name cannot be empty".to_string());
        }
        if truncate_at > MAX_TRUNCATE_WIDTH {
            return Err(format!("Column::new: truncation width {} exceeds maximum allowed ({})", 
                truncate_at, MAX_TRUNCATE_WIDTH));
        }

        let name_len = name.chars().count();
        let effective_truncate = if truncate_at > 0 {
            truncate_at.max(3).max(name_len)
        } else {
            truncate_at
        };
        
        Ok(Column {
            name,
            truncate_at: effective_truncate,
            justification,
            max_length: name_len,
        })
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
    fn format_cell(&self, cell_value: &str) -> Result<Vec<String>, String> {
        let lines: Vec<&str> = cell_value.split('\n').collect();
        if lines.len() > MAX_CELL_LINES {
            return Err(format!("Column::format_cell: number of lines ({}) exceeds maximum allowed ({})",
                lines.len(), MAX_CELL_LINES));
        }
        
        Ok(lines.into_iter()
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
            .collect())
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
    /// 
    /// # Errors
    /// - If columns vector is empty
    pub fn new(columns: Vec<Column>) -> Result<Self, String> {
        if columns.is_empty() {
            return Err("Table::new: table must have at least one column".to_string());
        }
        
        Ok(Table {
            columns,
            rows: Vec::new(),
        })
    }

    /// Adds a row to the table.
    /// 
    /// # Errors
    /// - If number of values doesn't match number of columns
    /// - If table would exceed MAX_TABLE_ROWS (5,000,000)
    /// - If any cell contains more than MAX_CELL_LINES (5000) lines
    pub fn add_row(&mut self, row: Vec<String>) -> Result<(), String> {
        if row.len() != self.columns.len() {
            return Err(format!(
                "Table::add_row: row has {} columns, expected {}",
                row.len(),
                self.columns.len()
            ));
        }

        if self.rows.len() >= MAX_TABLE_ROWS {
            return Err(format!(
                "Table::add_row: cannot add more rows, maximum ({}) reached",
                MAX_TABLE_ROWS
            ));
        }

        // Validate multiline limits before updating anything
        for value in &row {
            let line_count = value.split('\n').count();
            if line_count > MAX_CELL_LINES {
                return Err(format!(
                    "Table::add_row: cell contains {} lines, exceeding maximum allowed ({})",
                    line_count, MAX_CELL_LINES
                ));
            }
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
    
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.columns.is_empty() {
            return Ok(());
        }

        // Format header
        // Reminder: the [0] below is because the header is a single line, so we only need the first element of the vec returned by format_cell
        let header: Vec<String> = self.columns
            .iter()
            .map(|col| col.format_cell(&col.name).map_or_else(|e| e, |v| v[0].clone()))
            .collect();
        
        // Write the header to the formatter
        writeln!(f, "{}", header.join(" "))?;

        // Format separator between header and rows
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
        
        // Write the separator to the formatter
        writeln!(f, "{}", separator.join(" "))?;

        // Format rows with multiline support
        for row in &self.rows {
            
            // Convert each cell into a vector of formatted lines
            let formatted_cells: Vec<Vec<String>> = self.columns
                .iter()
                .zip(row)
                .map(|(col, value)| col.format_cell(value).map_or_else(|e| vec![e], |v| v))
                .collect();
            // Above creates a vec of vecs of strings, where each inner vec is a vec of strings representing the lines of a cell
            // It looks like this: [[line1, line2, line3], [line1, line2], [line1, line2, line3, line4]]
            // Non multiline cells are represented as a vec of one element

            // Find the maximum number of lines in any cell of this row
            let max_lines = formatted_cells
                .iter()
                .map(|cell| cell.len())
                .max()
                .unwrap_or(1);

            // Print each line of the row
            // For each line of the row, we need to print the corresponding line from each cell, or the empty string if the cell has fewer lines than the max
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
                
                // Write the line to the formatter
                writeln!(f, "{}", line.join(" "))?;
            }
        }

        // Return Ok to indicate successful formatting
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_table() {
        let mut table = Table::new(vec![
            Column::new("Name", 0, Justification::Left).unwrap(),
            Column::new("Age", 0, Justification::Left).unwrap(),
            Column::new("City", 0, Justification::Left).unwrap(),
        ]).unwrap();

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
            Column::new("Name", 10, Justification::Left).unwrap(),
            Column::new("Description", 35, Justification::Left).unwrap(),
        ]).unwrap();

        table.add_row(vec![
            "John Doe".to_string(),
            "A very long description that should be truncated".to_string(),
        ]).unwrap();

        println!("\n=== Truncation Test ===\n\n{}\n", table);
    }

    #[test]
    fn test_justification() {
        let cols = vec![
            Column::new("ID-R", 0, Justification::Right).unwrap(),
            Column::new("Name-L", 0, Justification::Left).unwrap(),
            Column::new("Balance-R", 0, Justification::Right).unwrap(),
        ];

        let mut table = Table::new(cols).unwrap();

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
            Column::new("Name", 0, Justification::Left).unwrap(),
            Column::new("Description", 0, Justification::Left).unwrap(),
            Column::new("Status", 0, Justification::Left).unwrap(),
        ]).unwrap();

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
            Column::new("Name", 0, Justification::Left).unwrap(),
            Column::new("Description", 35, Justification::Left).unwrap(),
            Column::new("Status", 0, Justification::Left).unwrap(),
            Column::new("Value Right1", 5, Justification::Right).unwrap(),
            Column::new("Value Right2", 0, Justification::Right).unwrap(),
        ]).unwrap();


        table.add_row(vec![
            "Jane Smith".to_string(),
            "Project Manager".to_string(),
            "On Leave".to_string(),
            "3000.00".to_string(),
            "4000.00".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "John Doe".to_string(),
            "Software Engineer to long to long to long to long".to_string(),
            "Active".to_string(),
            "1000.00".to_string(),
            "2000.00".to_string(),
        ]).unwrap();

        table.add_row(vec![
            "Bill Barney".to_string(),
            "Project Manager".to_string(),
            "On Leave".to_string(),
            "3040.00".to_string(),
            "4020.00".to_string(),
        ]).unwrap();
        
        table.add_row(vec![
            "Sally Christopher".to_string(),
            "Project Manager\nFormer engineer\nmore lines".to_string(),
            "On Leave".to_string(),
            "3040.00".to_string(),
            "4020.00".to_string(),
        ]).unwrap();
        
        table.add_row(vec![
            "Sofia Fraks".to_string(),
            "Director of Engineering ".to_string(),
            "Active".to_string(),
            "9000.00".to_string(),
            "9099.00".to_string(),
        ]).unwrap();
        
        println!("\n=== All Features Test ===\n\n{}\n", table);
    }
}
