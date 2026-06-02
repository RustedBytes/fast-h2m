#![allow(missing_docs)]

fn convert(
    html: &str,
    opts: Option<html_to_markdown_rs::ConversionOptions>,
) -> html_to_markdown_rs::error::Result<String> {
    html_to_markdown_rs::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use html_to_markdown_rs::ConversionOptions;

#[test]
fn test_basic_row_and_cell_conversion() {
    let html = r"<table>
    <row><cell>Header 1</cell><cell>Header 2</cell></row>
    <row><cell>Cell 1</cell><cell>Cell 2</cell></row>
    </table>";

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header 1"), "header 1 missing: {result}");
    assert!(result.contains("| Header 2"), "header 2 missing: {result}");
    assert!(result.contains("| Cell 1"), "cell 1 missing: {result}");
    assert!(result.contains("| Cell 2"), "cell 2 missing: {result}");
}

#[test]
fn test_cell_role_head_as_table_header() {
    let html = r#"<table>
    <row><cell role="head">Column 1</cell><cell role="head">Column 2</cell></row>
    <row><cell>Data 1</cell><cell>Data 2</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Column 1"), "column 1 missing: {result}");
    assert!(result.contains("| Column 2"), "column 2 missing: {result}");
    assert!(result.contains("| Data 1"), "data 1 missing: {result}");
    assert!(result.contains("| Data 2"), "data 2 missing: {result}");
    assert!(result.contains("| ---"), "separator row missing: {result}");
}

#[test]
fn test_mixed_html_and_xml_elements() {
    let html = r#"<table>
    <row><cell role="head">Name</cell><cell role="head">Age</cell></row>
    <row><td><strong>John</strong></td><td>25</td></row>
    <row><cell><em>Jane</em></cell><cell>30</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name"), "Name column missing: {result}");
    assert!(result.contains("| Age"), "Age column missing: {result}");
    assert!(result.contains("**John**"), "John bold missing: {result}");
    assert!(result.contains("*Jane*"), "Jane italic missing: {result}");
}

#[test]
fn test_tei_cols_and_rows_attributes() {
    let html = r#"<table cols="2" rows="3">
    <row><cell>Header 1</cell><cell>Header 2</cell></row>
    <row><cell>Cell 1</cell><cell>Cell 2</cell></row>
    <row><cell>Cell 3</cell><cell>Cell 4</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header 1"), "header 1 missing: {result}");
    assert!(result.contains("| Header 2"), "header 2 missing: {result}");
    assert!(result.contains("| Cell 1"), "cell 1 missing: {result}");
    assert!(result.contains("| Cell 2"), "cell 2 missing: {result}");
    assert!(result.contains("| Cell 3"), "cell 3 missing: {result}");
    assert!(result.contains("| Cell 4"), "cell 4 missing: {result}");
}

#[test]
fn test_graphic_element_with_xlink_href() {
    let html = r#"<table>
    <row>
        <cell role="head">Image</cell>
        <cell role="head">Description</cell>
    </row>
    <row>
        <cell><graphic xlink:href="image.png"/></cell>
        <cell>A test image</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Image"), "Image column missing: {result}");
    assert!(result.contains("| Description"), "Description column missing: {result}");
    assert!(result.contains("A test image"), "test image text missing: {result}");
    // graphic element handling may vary based on implementation
}

#[test]
fn test_graphic_in_table_cells() {
    let html = r#"<table>
    <row>
        <cell role="head">Figure</cell>
    </row>
    <row>
        <cell><graphic url="diagram.svg" alt="System Diagram"/></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Figure"), "Figure column missing: {result}");
}

#[test]
fn test_empty_cells_xml() {
    let html = r"<table>
    <row><cell>Data</cell><cell></cell></row>
    </table>";

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Data |  |"));
}

#[test]
fn test_nested_content_in_cells() {
    let html = r#"<table>
    <row><cell role="head">Text</cell><cell role="head">Formatted</cell></row>
    <row>
        <cell>Plain text</cell>
        <cell><b>Bold</b> and <a href="http://example.com">Link</a></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Text"), "Text column missing: {result}");
    assert!(result.contains("| Formatted"), "Formatted column missing: {result}");
    assert!(result.contains("| Plain text"), "plain text missing: {result}");
    assert!(result.contains("**Bold**"), "bold missing: {result}");
    assert!(result.contains("[Link](http://example.com)"), "link missing: {result}");
}

#[test]
fn test_mixed_tr_and_row_in_same_table() {
    let html = r"<table>
    <tr><th>Col 1</th><th>Col 2</th></tr>
    <row><cell>Data 1</cell><cell>Data 2</cell></row>
    <tr><td>Data 3</td><td>Data 4</td></tr>
    </table>";

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Col 1"), "Col 1 missing: {result}");
    assert!(result.contains("| Col 2"), "Col 2 missing: {result}");
    assert!(result.contains("| Data 1"), "Data 1 missing: {result}");
    assert!(result.contains("| Data 2"), "Data 2 missing: {result}");
    assert!(result.contains("| Data 3"), "Data 3 missing: {result}");
    assert!(result.contains("| Data 4"), "Data 4 missing: {result}");
}

#[test]
fn test_cell_without_role_attribute_defaults_to_data() {
    let html = r"<table>
    <row><cell>Header</cell></row>
    <row><cell>Data Cell</cell></row>
    </table>";

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header"), "Header missing: {result}");
    assert!(result.contains("| Data Cell"), "Data Cell missing: {result}");
}

#[test]
fn test_xml_table_with_multiline_content() {
    let html = r#"<table>
    <row><cell role="head">Content</cell></row>
    <row>
        <cell>
            <p>Line 1</p>
            <p>Line 2</p>
        </cell>
    </row>
    </table>"#;

    let options = ConversionOptions {
        br_in_tables: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).unwrap();

    assert!(result.contains("| Content"), "Content column missing: {result}");
    assert!(result.contains("Line 1"), "Line 1 missing: {result}");
    assert!(result.contains("Line 2"), "Line 2 missing: {result}");
}

#[test]
fn test_cell_with_lists() {
    let html = r#"<table>
    <row><cell role="head">Items</cell></row>
    <row>
        <cell>
            <ul>
                <li>Item 1</li>
                <li>Item 2</li>
            </ul>
        </cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Items"), "Items column missing: {result}");
    assert!(result.contains("Item 1"), "Item 1 missing: {result}");
    assert!(result.contains("Item 2"), "Item 2 missing: {result}");
}

#[test]
fn test_single_column_xml_table() {
    let html = r#"<table>
    <row><cell role="head">Header</cell></row>
    <row><cell>Data 1</cell></row>
    <row><cell>Data 2</cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header"), "Header missing: {result}");
    assert!(result.contains("| ---"), "separator row missing: {result}");
    assert!(result.contains("| Data 1"), "Data 1 missing: {result}");
    assert!(result.contains("| Data 2"), "Data 2 missing: {result}");
}

#[test]
fn test_cell_with_code_blocks() {
    let html = r#"<table>
    <row><cell role="head">Code</cell></row>
    <row><cell><code>function()</code></cell></row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Code"), "Code column missing: {result}");
    assert!(result.contains("`function()`"), "code block missing: {result}");
}

#[test]
fn test_xml_table_with_emphasis() {
    let html = r#"<table>
    <row>
        <cell role="head"><em>Emphasized</em></cell>
        <cell role="head"><strong>Strong</strong></cell>
    </row>
    <row>
        <cell><i>Italic</i></cell>
        <cell><b>Bold</b></cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("*Emphasized*"));
    assert!(result.contains("**Strong**"));
    assert!(result.contains("*Italic*"));
    assert!(result.contains("**Bold**"));
}

#[test]
fn test_xml_table_with_multiple_headers() {
    let html = r#"<table>
    <row>
        <cell role="head">First</cell>
        <cell role="head">Second</cell>
        <cell role="head">Third</cell>
    </row>
    <row>
        <cell>A</cell>
        <cell>B</cell>
        <cell>C</cell>
    </row>
    <row>
        <cell>D</cell>
        <cell>E</cell>
        <cell>F</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| First"), "First column missing: {result}");
    assert!(result.contains("| Second"), "Second column missing: {result}");
    assert!(result.contains("| Third"), "Third column missing: {result}");
    assert!(result.contains("| ---"), "separator row missing: {result}");
    assert!(result.contains("| A"), "A missing: {result}");
    assert!(result.contains("| B"), "B missing: {result}");
    assert!(result.contains("| C"), "C missing: {result}");
    assert!(result.contains("| D"), "D missing: {result}");
    assert!(result.contains("| E"), "E missing: {result}");
    assert!(result.contains("| F"), "F missing: {result}");
}

#[test]
fn test_cell_role_variations() {
    let html = r#"<table>
    <row>
        <cell role="head">Header Cell</cell>
        <cell role="data">Data Cell</cell>
    </row>
    <row>
        <cell role="label">Label</cell>
        <cell role="head">Another Header</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Header Cell"), "Header Cell missing: {result}");
    assert!(result.contains("| Data Cell"), "Data Cell missing: {result}");
    assert!(result.contains("| Label"), "Label missing: {result}");
    assert!(result.contains("| Another Header"), "Another Header missing: {result}");
}

#[test]
fn test_deeply_nested_xml_content() {
    let html = r#"<table>
    <row><cell role="head">Complex</cell></row>
    <row>
        <cell>
            <div>
                <p>
                    <strong>
                        <em>Nested</em>
                    </strong>
                </p>
            </div>
        </cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Complex"), "Complex column missing: {result}");
    assert!(result.contains("Nested"), "Nested text missing: {result}");
}

#[test]
fn test_xml_table_with_attributes_preserved() {
    let html = r#"<table id="table1" class="data-table" xmlns:tei="http://www.tei-c.org/ns/1.0">
    <row>
        <cell role="head">Column 1</cell>
        <cell role="head">Column 2</cell>
    </row>
    <row>
        <cell>Value 1</cell>
        <cell>Value 2</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Column 1"), "Column 1 missing: {result}");
    assert!(result.contains("| Column 2"), "Column 2 missing: {result}");
    assert!(result.contains("| Value 1"), "Value 1 missing: {result}");
    assert!(result.contains("| Value 2"), "Value 2 missing: {result}");
}

#[test]
fn test_mixed_cell_types_in_rows() {
    let html = r#"<table>
    <row>
        <cell role="head">Name</cell>
        <th>Age</th>
        <cell role="head">City</cell>
    </row>
    <row>
        <cell>John</cell>
        <td>25</td>
        <cell>NYC</cell>
    </row>
    </table>"#;

    let result = convert(html, None).unwrap();
    assert!(result.contains("| Name"), "Name column missing: {result}");
    assert!(result.contains("| Age"), "Age column missing: {result}");
    assert!(result.contains("| City"), "City column missing: {result}");
    assert!(result.contains("| John"), "John missing: {result}");
    assert!(result.contains("| 25"), "25 missing: {result}");
    assert!(result.contains("| NYC"), "NYC missing: {result}");
}
