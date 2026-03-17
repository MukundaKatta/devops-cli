use tabled::{
    settings::{object::Rows, Alignment, Modify, Style},
    Table, Tabled,
};

/// Build a pretty table from a vector of Tabled items.
pub fn pretty_table<T: Tabled>(data: &[T]) -> String {
    Table::new(data)
        .with(Style::rounded())
        .with(Modify::new(Rows::first()).with(Alignment::center()))
        .to_string()
}

/// Build a simple two-column key-value table.
pub fn kv_table(rows: &[(&str, &str)]) -> String {
    use tabled::builder::Builder;

    let mut builder = Builder::default();
    builder.push_record(["Key", "Value"]);
    for (k, v) in rows {
        builder.push_record([*k, *v]);
    }
    builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Rows::first()).with(Alignment::center()))
        .to_string()
}

/// Build a table from raw rows (vec of vec of strings) with headers.
pub fn raw_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    use tabled::builder::Builder;

    let mut builder = Builder::default();
    builder.push_record(headers.iter().map(|h| h.to_string()));
    for row in rows {
        builder.push_record(row.iter().map(|c| c.to_string()));
    }
    builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Rows::first()).with(Alignment::center()))
        .to_string()
}
