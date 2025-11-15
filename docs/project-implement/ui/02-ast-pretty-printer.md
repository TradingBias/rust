# AST Pretty Printer Implementation

## Overview

The AST Pretty Printer converts the internal Abstract Syntax Tree (AST) representation of strategies into human-readable formula strings for display in the UI. This is essential for users to understand and interpret evolved strategies.

---

## Module Location

**File**: `src/engines/generation/ast.rs` (extend existing)

**Or New File**: `src/engines/generation/formatter.rs`

---

## Requirements

### Input
- `AstNode` from `src/types.rs`

### Output
- Human-readable string representation
- Examples:
  - `"AND(>(RSI(14), 30), <(RSI(14), 70))"`
  - `"AND(>(EMA(12), SMA(26)), >(Volume, 1000000))"`
  - `">(MACD(12, 26, 9), 0)"`

### Features
1. **Nested Formatting**: Handle deeply nested expressions
2. **Indentation**: Optional multi-line output with indentation
3. **Parameter Display**: Show indicator parameters inline
4. **Type Awareness**: Different formatting for primitives vs indicators
5. **Truncation**: Support truncating long formulas for table display

---

## Implementation

### 1. Core Formatter (`src/engines/generation/formatter.rs`)

```rust
use crate::types::{AstNode, Value};

pub struct FormulaFormatter {
    config: FormatterConfig,
}

#[derive(Clone)]
pub struct FormatterConfig {
    /// Use multi-line output with indentation
    pub multiline: bool,
    /// Number of spaces per indent level
    pub indent_size: usize,
    /// Maximum line length before wrapping (if multiline)
    pub max_line_length: usize,
    /// Truncate output if longer than this (0 = no truncation)
    pub max_length: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            multiline: false,
            indent_size: 2,
            max_line_length: 80,
            max_length: 0, // No truncation
        }
    }
}

impl FormatterConfig {
    /// Configuration for table display (single line, truncated)
    pub fn table_display() -> Self {
        Self {
            multiline: false,
            max_length: 60,
            ..Default::default()
        }
    }

    /// Configuration for detailed view (multi-line, no truncation)
    pub fn detailed_view() -> Self {
        Self {
            multiline: true,
            max_length: 0,
            ..Default::default()
        }
    }

    /// Configuration for compact single line
    pub fn compact() -> Self {
        Self {
            multiline: false,
            max_length: 0,
            ..Default::default()
        }
    }
}

impl FormulaFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self { config }
    }

    /// Format AstNode to string
    pub fn format(&self, node: &AstNode) -> String {
        let formatted = if self.config.multiline {
            self.format_multiline(node, 0)
        } else {
            self.format_inline(node)
        };

        if self.config.max_length > 0 && formatted.len() > self.config.max_length {
            let truncated = &formatted[..self.config.max_length - 3];
            format!("{}...", truncated)
        } else {
            formatted
        }
    }

    /// Format as single line
    fn format_inline(&self, node: &AstNode) -> String {
        match node {
            AstNode::Const(value) => self.format_value(value),
            AstNode::Call { function, args } => {
                let formatted_args: Vec<String> = args.iter()
                    .map(|arg| self.format_inline(arg))
                    .collect();
                format!("{}({})", function, formatted_args.join(", "))
            }
            AstNode::Rule { condition, action } => {
                format!(
                    "IF {} THEN {}",
                    self.format_inline(condition),
                    self.format_inline(action)
                )
            }
        }
    }

    /// Format with indentation and line breaks
    fn format_multiline(&self, node: &AstNode, indent_level: usize) -> String {
        let indent = " ".repeat(indent_level * self.config.indent_size);
        let next_indent = " ".repeat((indent_level + 1) * self.config.indent_size);

        match node {
            AstNode::Const(value) => self.format_value(value),
            AstNode::Call { function, args } => {
                if args.is_empty() {
                    return format!("{}()", function);
                }

                // Check if inline version fits
                let inline = self.format_inline(node);
                if inline.len() <= self.config.max_line_length {
                    return inline;
                }

                // Multi-line format
                let formatted_args: Vec<String> = args.iter()
                    .map(|arg| format!("{}{}", next_indent, self.format_multiline(arg, indent_level + 1)))
                    .collect();

                format!(
                    "{}(\n{}\n{})",
                    function,
                    formatted_args.join(",\n"),
                    indent
                )
            }
            AstNode::Rule { condition, action } => {
                format!(
                    "IF {} THEN {}",
                    self.format_multiline(condition, indent_level),
                    self.format_multiline(action, indent_level)
                )
            }
        }
    }

    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => {
                // Format with appropriate precision
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else if f.abs() < 0.01 {
                    format!("{:.4}", f)
                } else {
                    format!("{:.2}", f)
                }
            }
            Value::String(s) => format!("\"{}\"", s),
            Value::Bool(b) => b.to_string(),
        }
    }
}

/// Convenience functions for common use cases
impl AstNode {
    /// Format as compact single-line string
    pub fn to_formula(&self) -> String {
        let formatter = FormulaFormatter::new(FormatterConfig::compact());
        formatter.format(self)
    }

    /// Format for table display (truncated)
    pub fn to_formula_short(&self) -> String {
        let formatter = FormulaFormatter::new(FormatterConfig::table_display());
        formatter.format(self)
    }

    /// Format with indentation for detailed view
    pub fn to_formula_pretty(&self) -> String {
        let formatter = FormulaFormatter::new(FormatterConfig::detailed_view());
        formatter.format(self)
    }
}
```

---

### 2. Alternative: Simple Implementation (Minimal)

If the full formatter is too complex for initial implementation, here's a simpler version:

```rust
// Add to src/types.rs or src/engines/generation/ast.rs

impl AstNode {
    pub fn to_formula(&self) -> String {
        match self {
            AstNode::Const(value) => match value {
                Value::Integer(i) => i.to_string(),
                Value::Float(f) => format!("{:.2}", f),
                Value::String(s) => format!("\"{}\"", s),
                Value::Bool(b) => b.to_string(),
            },
            AstNode::Call { function, args } => {
                let formatted_args: Vec<String> = args.iter()
                    .map(|arg| arg.to_formula())
                    .collect();
                format!("{}({})", function, formatted_args.join(", "))
            }
            AstNode::Rule { condition, action } => {
                format!("IF {} THEN {}", condition.to_formula(), action.to_formula())
            }
        }
    }

    pub fn to_formula_truncated(&self, max_len: usize) -> String {
        let full = self.to_formula();
        if full.len() > max_len {
            format!("{}...", &full[..max_len - 3])
        } else {
            full
        }
    }
}
```

---

## Usage Examples

### Example 1: Simple Expression

```rust
use crate::types::{AstNode, Value};

// AST: >(RSI(14), 30)
let ast = AstNode::Call {
    function: "Greater".to_string(),
    args: vec![
        Box::new(AstNode::Call {
            function: "RSI".to_string(),
            args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
        }),
        Box::new(AstNode::Const(Value::Integer(30))),
    ],
};

println!("{}", ast.to_formula());
// Output: "Greater(RSI(14), 30)"
```

### Example 2: Complex Nested Expression

```rust
// AST: AND(>(RSI(14), 30), <(RSI(14), 70), >(EMA(12), SMA(26)))
let complex_ast = AstNode::Call {
    function: "And".to_string(),
    args: vec![
        Box::new(AstNode::Call {
            function: "Greater".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "RSI".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                }),
                Box::new(AstNode::Const(Value::Integer(30))),
            ],
        }),
        Box::new(AstNode::Call {
            function: "Less".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "RSI".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                }),
                Box::new(AstNode::Const(Value::Integer(70))),
            ],
        }),
        Box::new(AstNode::Call {
            function: "Greater".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "EMA".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(12)))],
                }),
                Box::new(AstNode::Call {
                    function: "SMA".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(26)))],
                }),
            ],
        }),
    ],
};

// Compact format
println!("{}", complex_ast.to_formula());
// Output: "And(Greater(RSI(14), 30), Less(RSI(14), 70), Greater(EMA(12), SMA(26)))"

// Pretty format
println!("{}", complex_ast.to_formula_pretty());
// Output:
// And(
//   Greater(RSI(14), 30),
//   Less(RSI(14), 70),
//   Greater(EMA(12), SMA(26))
// )

// Table display format
println!("{}", complex_ast.to_formula_short());
// Output: "And(Greater(RSI(14), 30), Less(RSI(14), 70), Gre..."
```

---

## Integration with UI

### In Strategy Display Model (`ui/models/strategy_display.rs`)

```rust
use crate::engines::generation::hall_of_fame::EliteStrategy;
use crate::types::AstNode;

#[derive(Clone)]
pub struct StrategyDisplay {
    pub rank: usize,
    pub fitness: f64,
    pub return_pct: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub formula: String,          // Short version for table
    pub formula_full: String,      // Full version for details
    pub formula_pretty: String,    // Pretty-printed for copy-paste
    pub equity_curve: Vec<f64>,
    pub trades: Vec<Trade>,
}

impl From<EliteStrategy> for StrategyDisplay {
    fn from(elite: EliteStrategy) -> Self {
        let ast = &elite.result.ast;

        Self {
            rank: 0, // Set by UI
            fitness: elite.fitness,
            return_pct: elite.result.metrics.get("return_pct").copied().unwrap_or(0.0),
            total_trades: elite.result.trades.len(),
            win_rate: calculate_win_rate(&elite.result.trades),
            max_drawdown: elite.result.metrics.get("max_drawdown").copied().unwrap_or(0.0),
            sharpe_ratio: elite.result.metrics.get("sharpe_ratio").copied().unwrap_or(0.0),
            formula: ast.to_formula_short(),           // Truncated for table
            formula_full: ast.to_formula(),            // Full single-line
            formula_pretty: ast.to_formula_pretty(),   // Multi-line indented
            equity_curve: elite.result.equity_curve.clone(),
            trades: elite.result.trades.clone(),
        }
    }
}
```

### In Main Panel Widget (`ui/panels/main_panel.rs`)

```rust
// Display truncated formula in table cell
ui.label(&strategy.formula);

// Tooltip shows full formula on hover
if ui.add(egui::Label::new(&strategy.formula).sense(egui::Sense::hover())).hovered() {
    egui::show_tooltip(ui.ctx(), egui::Id::new("formula_tooltip"), |ui| {
        ui.label(&strategy.formula_full);
    });
}
```

### In Right Panel Widget (`ui/panels/right_panel.rs`)

```rust
// Show pretty-printed formula in detailed view
egui::CollapsingHeader::new("Strategy Formula").default_open(true).show(ui, |ui| {
    ui.add(
        egui::TextEdit::multiline(&mut strategy.formula_pretty.as_str())
            .code_editor()
            .desired_width(f32::INFINITY)
    );

    if ui.button("Copy Formula").clicked() {
        ui.output_mut(|o| o.copied_text = strategy.formula_full.clone());
    }
});
```

---

## Enhanced Features (Optional)

### 1. Syntax Highlighting

Add color coding for different node types:

```rust
pub struct SyntaxHighlighter {
    colors: SyntaxColors,
}

pub struct SyntaxColors {
    pub function: Color32,
    pub parameter: Color32,
    pub operator: Color32,
    pub constant: Color32,
}

impl SyntaxHighlighter {
    pub fn format_rich_text(&self, node: &AstNode) -> egui::RichText {
        // Return colored RichText for egui display
    }
}
```

### 2. LaTeX Output

For academic papers or documentation:

```rust
impl AstNode {
    pub fn to_latex(&self) -> String {
        match self {
            AstNode::Call { function: "Greater", args } => {
                format!(r"{} > {}", args[0].to_latex(), args[1].to_latex())
            }
            AstNode::Call { function: "RSI", args } => {
                format!(r"\text{{RSI}}_{{{}}}", args[0].to_latex())
            }
            // ... more LaTeX formatting
        }
    }
}
```

### 3. Human-Readable Operators

Map internal function names to readable operators:

```rust
fn get_operator_symbol(function: &str) -> Option<&'static str> {
    match function {
        "Greater" | ">" => Some(">"),
        "Less" | "<" => Some("<"),
        "GreaterEqual" | ">=" => Some(">="),
        "LessEqual" | "<=" => Some("<="),
        "Equal" | "==" => Some("=="),
        "And" | "&&" => Some("AND"),
        "Or" | "||" => Some("OR"),
        _ => None,
    }
}

// Usage in formatter:
if let Some(op) = get_operator_symbol(function) {
    if args.len() == 2 {
        // Infix notation
        return format!("({} {} {})",
            self.format_inline(&args[0]),
            op,
            self.format_inline(&args[1])
        );
    }
}
```

Example output: `"(RSI(14) > 30 AND RSI(14) < 70)"`

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AstNode, Value};

    #[test]
    fn test_simple_constant() {
        let ast = AstNode::Const(Value::Float(3.14));
        assert_eq!(ast.to_formula(), "3.14");
    }

    #[test]
    fn test_simple_call() {
        let ast = AstNode::Call {
            function: "RSI".to_string(),
            args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
        };
        assert_eq!(ast.to_formula(), "RSI(14)");
    }

    #[test]
    fn test_nested_call() {
        let ast = AstNode::Call {
            function: "Greater".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "RSI".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                }),
                Box::new(AstNode::Const(Value::Integer(30))),
            ],
        };
        assert_eq!(ast.to_formula(), "Greater(RSI(14), 30)");
    }

    #[test]
    fn test_truncation() {
        let long_ast = create_deeply_nested_ast(10); // Helper to create deep tree
        let short = long_ast.to_formula_short();
        assert!(short.len() <= 63); // 60 + "..."
        assert!(short.ends_with("...") || short.len() <= 60);
    }

    #[test]
    fn test_multiline_format() {
        let ast = create_complex_ast(); // Helper
        let pretty = ast.to_formula_pretty();
        assert!(pretty.contains('\n')); // Should have newlines
        assert!(pretty.contains("  ")); // Should have indentation
    }
}
```

---

## Performance Considerations

1. **Caching**: For large Hall of Fame, cache formatted strings in `StrategyDisplay`
2. **Lazy Evaluation**: Only format visible strategies in UI table
3. **String Allocation**: Use `String::with_capacity()` for large formulas

---

## Recommended Approach

### Phase 1 (MVP)
Implement the simple version (just `to_formula()` and `to_formula_truncated()`) as a method on `AstNode` in `src/types.rs`.

### Phase 2 (Enhancement)
Extract to dedicated `formatter.rs` module with configuration options.

### Phase 3 (Polish)
Add syntax highlighting, operator symbols, and LaTeX export.

---

## Summary

The AST Pretty Printer is essential for:
- ✅ Displaying strategies in the UI table
- ✅ Showing detailed formulas in the right panel
- ✅ Allowing users to copy/export strategies
- ✅ Debugging and understanding evolved strategies

**Recommended Implementation**: Start with simple version (50 lines) in Phase 1, then enhance in Phase 4 (Polish).
