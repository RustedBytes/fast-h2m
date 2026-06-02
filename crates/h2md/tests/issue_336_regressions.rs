#![allow(missing_docs)]

//! Regression tests for issue #336: content after a table is silently truncated when an
//! unclosed `<p>` appears inside a `<td>` without a closing `</p>`.
//!
//! The `tl` parser mishandles malformed Word/Outlook HTML (produced by Microsoft Word's
//! HTML export) where a `<p>` tag inside a `<td>` is not properly closed before the
//! `</td>` end tag.  When the unclosed `<p>` appears in a table cell, `tl` treats all
//! subsequent sibling elements — including other `<td>` cells and all post-table content —
//! as descendants of that `<p>`.
//!
//! The most visible symptom: any `<p>`, `<h2>`, or other element that comes **after** the
//! closing `</table>` tag gets swallowed into the last table cell and is never emitted as
//! top-level document content, causing the output to appear truncated.
//!
//! Triggered by the `<td><p class='MsoNormal'>&nbsp;</td>` pattern that appears 34 times
//! in `ms-fscc.html` — specifically when that pattern is the *first* `<td>` in a row.

fn convert(html: &str) -> String {
    h2md::convert(html, None)
        .expect("conversion should not fail")
        .content
        .unwrap_or_default()
}

/// Minimal reproduction: a table with two rows each having an unclosed `<p>` as the first
/// `<td>`.  The `<p>SectionAfterTable</p>` that follows MUST appear in the output.
///
/// Without the fix, `tl` nests the post-table `<p>` inside the last table `<td>`, so the
/// converter either drops it (blank-table fast-return) or renders it inline inside a table
/// cell — in both cases it is absent as a top-level paragraph.
#[test]
fn test_content_after_table_not_dropped_when_td_has_unclosed_p() {
    let html = r"<table>
 <tr>
  <td width=1><p class='MsoNormal'>&nbsp;</td>
  <td>Row1Data</td>
 </tr>
 <tr>
  <td width=1><p class='MsoNormal'>&nbsp;</td>
  <td>Row2Data</td>
 </tr>
</table>
<p>SectionAfterTable</p>";

    let result = convert(html);
    assert!(
        result.contains("SectionAfterTable"),
        "Content after the table must not be swallowed into the table DOM. Got:\n{result}"
    );
}

/// Headings and subsequent sections after such a table must also be preserved.
#[test]
fn test_headings_after_table_with_msonormal_p_in_td_are_preserved() {
    let html = r"<table>
 <tr>
  <td width=1><p class='MsoNormal'>&nbsp;</td>
  <td>Some data</td>
 </tr>
</table>
<h2>Section 2.4.22</h2>
<p>This section describes something important.</p>";

    let result = convert(html);
    assert!(
        result.contains("Section 2.4.22"),
        "Heading after the table must appear in the output. Got:\n{result}"
    );
    assert!(
        result.contains("This section describes something important"),
        "Paragraph after the heading must appear. Got:\n{result}"
    );
}
