# Rust Table Formatter

A flexible Rust library for formatting tabular data with support for:
- Multi-line cell content
- Custom column widths and truncation
- Left/right text justification
- Automatic column width adjustment

## Usage

In Cargo.toml:
```toml
[dependencies]
ctable = { git = "https://github.com/dcopenhaver/ctable-rs" }
```

### Basic Example

```rust
use ctable::{Table, Column, Justification};

// Create a new table with 5 columns
let mut table = Table::new(vec![
    Column::new("Name", 0, Justification::Left).unwrap(),
    Column::new("Description", 35, Justification::Left).unwrap(),
    Column::new("Status", 0, Justification::Left).unwrap(),
    Column::new("Value Right1", 5, Justification::Right).unwrap(),
    Column::new("Value Right2", 0, Justification::Right).unwrap(),
]).unwrap();

// Add rows (usually from inside a loop of some kind processing data)
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

// display the table
println!("{}", table);
```

#### Output

```
Name              Description                         Status   Value Right1 Value Right2
----------------- ----------------------------------- -------- ------------ ------------
Jane Smith        Project Manager                     On Leave      3000.00      4000.00
John Doe          Software Engineer to long to lon... Active        1000.00      2000.00
Bill Barney       Project Manager                     On Leave      3040.00      4020.00
Sally Christopher Project Manager                     On Leave      3040.00      4020.00
                  Former engineer
                  more lines
Sofia Fraks       Director of Engineering             Active        9000.00      9099.00
```