// Template Method pattern: exporting a sales report. The overall recipe is
// always the same (title, header row, one line per sale, total line), but
// each output format fills in those steps differently.
//
// In Rust, the "template method" is a trait method with a default body that
// calls other trait methods. Each format implements only the steps it needs.

struct Sale {
    product: String,
    quantity: u32,
    unit_price_cents: u32,
}

impl Sale {
    fn new(product: &str, quantity: u32, unit_price_cents: u32) -> Sale {
        Sale {
            product: product.to_string(),
            quantity,
            unit_price_cents,
        }
    }

    fn total_cents(&self) -> u64 {
        u64::from(self.quantity) * u64::from(self.unit_price_cents)
    }
}

fn euros(cents: u64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

trait ReportExporter {
    // ---- The template method ------------------------------------------------
    //
    // These formats reuse this default recipe. It decides the order of the
    // steps and does the shared work (looping, adding up the total).
    // Rust also allows implementors to override this default method.

    fn export(&self, title: &str, sales: &[Sale]) -> String {
        let mut output = String::new();
        output.push_str(&self.title(title));
        output.push_str(&self.header());
        for sale in sales {
            output.push_str(&self.row(sale));
        }
        let total = sales
            .iter()
            .map(Sale::total_cents)
            .try_fold(0_u64, u64::checked_add)
            .expect("report total exceeds u64 cents");
        output.push_str(&self.footer(total));
        output
    }

    // ---- Steps every format must provide (no default) ------------------------

    fn header(&self) -> String;
    fn row(&self, sale: &Sale) -> String;

    // ---- Optional steps (a "hook" with a default that can be overridden) ----

    /// Most formats show the title as a plain line.
    fn title(&self, title: &str) -> String {
        format!("{title}\n")
    }

    /// By default there's no total line.
    fn footer(&self, _total_cents: u64) -> String {
        String::new()
    }
}

// ---- Concrete formats ---------------------------------------------------------

/// Comma-separated values, for spreadsheets. Spreadsheets don't want a title
/// line, so it overrides `title` to produce nothing.
struct CsvExporter;

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

impl ReportExporter for CsvExporter {
    fn title(&self, _title: &str) -> String {
        String::new()
    }

    fn header(&self) -> String {
        "product,quantity,unit_price,total\n".to_string()
    }

    fn row(&self, sale: &Sale) -> String {
        format!(
            "{},{},{},{}\n",
            csv_field(&sale.product),
            sale.quantity,
            euros(u64::from(sale.unit_price_cents)),
            euros(sale.total_cents())
        )
    }
}

/// A Markdown table, for documentation or chat. Overrides `title` to make a
/// heading and `footer` to add a total row.
struct MarkdownExporter;

fn markdown_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        let escaped = match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '|' => "&#124;",
            '*' => "&#42;",
            '_' => "&#95;",
            '~' => "&#126;",
            '[' => "&#91;",
            ']' => "&#93;",
            '`' => "&#96;",
            '\\' => "&#92;",
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                "<br>"
            }
            '\n' => "<br>",
            _ => {
                output.push(c);
                continue;
            }
        };
        output.push_str(escaped);
    }
    output
}

impl ReportExporter for MarkdownExporter {
    fn title(&self, title: &str) -> String {
        format!("## {}\n\n", markdown_text(title))
    }

    fn header(&self) -> String {
        "| Product | Qty | Unit price | Total |\n|---|---:|---:|---:|\n".to_string()
    }

    fn row(&self, sale: &Sale) -> String {
        format!(
            "| {} | {} | {} € | {} € |\n",
            markdown_text(&sale.product),
            sale.quantity,
            euros(u64::from(sale.unit_price_cents)),
            euros(sale.total_cents())
        )
    }

    fn footer(&self, total_cents: u64) -> String {
        format!("| **Total** | | | **{} €** |\n", euros(total_cents))
    }
}

/// Fixed-width columns for a terminal. Uses the default title, and adds a
/// separator line and total.
struct PlainTextExporter;

impl ReportExporter for PlainTextExporter {
    fn header(&self) -> String {
        format!("{:<12} {:>4} {:>10}\n", "Product", "Qty", "Total")
    }

    fn row(&self, sale: &Sale) -> String {
        format!(
            "{:<12} {:>4} {:>10}\n",
            sale.product,
            sale.quantity,
            euros(sale.total_cents())
        )
    }

    fn footer(&self, total_cents: u64) -> String {
        format!(
            "{}\n{:<12} {:>15}\n",
            "-".repeat(28),
            "Total",
            euros(total_cents)
        )
    }
}

fn main() {
    let sales = vec![
        Sale::new("Coffee", 120, 250),
        Sale::new("Croissant", 45, 180),
        Sale::new("Tea", 60, 200),
    ];

    // The same call, three different results. `&dyn ReportExporter` means
    // "any exporter", chosen at runtime.
    let exporters: [(&str, &dyn ReportExporter); 3] = [
        ("CSV", &CsvExporter),
        ("Markdown", &MarkdownExporter),
        ("Plain text", &PlainTextExporter),
    ];

    for (name, exporter) in exporters {
        println!("===== {name} =====");
        print!("{}", exporter.export("Café sales, Monday", &sales));
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sales() -> Vec<Sale> {
        vec![Sale::new("A", 2, 150), Sale::new("B", 1, 300)]
    }

    #[test]
    fn csv_has_no_title_and_no_total() {
        let output = CsvExporter.export("Ignored", &sales());
        assert_eq!(
            output,
            "product,quantity,unit_price,total\nA,2,1.50,3.00\nB,1,3.00,3.00\n"
        );
    }

    #[test]
    fn markdown_has_heading_and_total_row() {
        let output = MarkdownExporter.export("Report", &sales());
        assert!(output.starts_with("## Report\n"));
        assert!(output.ends_with("| **Total** | | | **6.00 €** |\n"));
    }

    #[test]
    fn every_format_follows_the_same_order() {
        // These exporters reuse the default recipe and preserve row order.
        for exporter in [
            &CsvExporter as &dyn ReportExporter,
            &MarkdownExporter,
            &PlainTextExporter,
        ] {
            let output = exporter.export("T", &sales());
            let a = output.find("A").unwrap();
            let b = output.find("B").unwrap();
            assert!(a < b, "rows must keep their order");
        }
    }

    #[test]
    fn empty_report_still_has_header() {
        let output = PlainTextExporter.export("Empty", &[]);
        assert!(output.contains("Product"));
        assert!(output.contains("0.00"));
    }
    #[test]
    fn csv_quotes_delimiters_quotes_and_newlines() {
        let sale = Sale::new("Tea, \"iced\"\nlarge", 1, 100);
        assert_eq!(
            CsvExporter.row(&sale),
            "\"Tea, \"\"iced\"\"\nlarge\",1,1.00,1.00\n"
        );
        assert_eq!(
            Sale::new("A", u32::MAX, u32::MAX).total_cents(),
            u64::from(u32::MAX).pow(2)
        );
    }

    #[test]
    fn markdown_names_keep_their_text_and_one_table_cell() {
        let row = MarkdownExporter.row(&Sale::new("A|B\n<script>*x*", 1, 100));
        assert_eq!(row.matches('|').count(), 5);
        assert!(row.contains("A&#124;B<br>&lt;script&gt;&#42;x&#42;"));
        assert_eq!(
            markdown_text("~~sale~~\r\n"),
            "&#126;&#126;sale&#126;&#126;<br>"
        );
    }
}
