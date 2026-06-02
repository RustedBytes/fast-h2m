//! Criterion benchmarks for representative `fast_h2m::convert` workloads.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use fast_h2m::{convert, ConversionOptions, TierStrategy};

const SIMPLE_HTML: &str = r#"
<article>
  <h1>Zero-copy conversion</h1>
  <p>This benchmark measures a simple document with <strong>inline</strong>
  formatting, <a href="https://example.com">links</a>, and escaped entities like
  &amp; and &quot;quotes&quot;.</p>
  <ul>
    <li>First item</li>
    <li>Second item with <em>emphasis</em></li>
    <li>Third item with <code>inline_code()</code></li>
  </ul>
</article>
"#;

const TABLE_HTML: &str = r#"
<table>
  <thead>
    <tr><th>Name</th><th>Language</th><th>Notes</th></tr>
  </thead>
  <tbody>
    <tr><td>fast_h2m</td><td>Rust</td><td>HTML to Markdown conversion</td></tr>
    <tr><td>criterion</td><td>Rust</td><td>Statistical benchmarking</td></tr>
    <tr><td>astral-tl</td><td>Rust</td><td>HTML parser</td></tr>
  </tbody>
</table>
"#;

const INLINE_HEAVY_HTML: &str = r#"
<p>Inline formatting mixes <strong>bold</strong>, <em>emphasis</em>,
<code>code()</code>, <a href="https://example.com/docs">links</a>,
<mark>marks</mark>, <sub>sub</sub>, and <sup>sup</sup> in one paragraph.</p>
"#;

const LIST_HTML: &str = r#"
<ol start="3">
  <li>Install Rust</li>
  <li>Run cargo test</li>
  <li>Compare benchmark output</li>
</ol>
<ul>
  <li>Small documents</li>
  <li>Medium documents</li>
  <li>Real-world fixtures</li>
</ul>
"#;

const MEDIA_HTML: &str = r#"
<figure>
  <img src="https://example.com/chart.png" alt="Conversion chart">
  <figcaption>Images should keep useful alternate text and captions.</figcaption>
</figure>
"#;

const SCRIPT_STYLE_HTML: &str = r#"
<style>.hidden { display: none }</style>
<script>window.__bench__ = "<div>not content</div>";</script>
<article><p>Visible content after script and style stripping.</p></article>
"#;

const CUSTOM_ELEMENT_HTML: &str = r#"
<article-card>
  <h2>Custom element fallback</h2>
  <p>Custom elements intentionally route through Tier-2.</p>
</article-card>
"#;

const ENTITY_HEAVY_HTML: &str =
    "<p>&amp; &quot;quoted&quot; &#x1F680; &lt;escaped&gt; &nbsp; repeated &amp; text</p>";

const TEXT_WITH_NEWLINES: &str = "Plain text\nwith multiple\nlines and no HTML tags.";

const HACKER_NEWS_FIXTURE: &str =
    include_str!("../../../test_documents/html/issues/gh-121-hacker-news.html");

const WIKIPEDIA_SMALL_FIXTURE: &str =
    include_str!("../../../test_documents/html/wikipedia/small_html.html");

fn options_for(strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        tier_strategy: strategy,
        ..ConversionOptions::default()
    }
}

fn bench_convert_cases(c: &mut Criterion) {
    let cases = [
        ("plain_text", "Just text with no HTML tags."),
        ("text_with_newlines", TEXT_WITH_NEWLINES),
        ("simple_html", SIMPLE_HTML),
        ("inline_heavy_html", INLINE_HEAVY_HTML),
        ("list_html", LIST_HTML),
        ("media_html", MEDIA_HTML),
        ("script_style_html", SCRIPT_STYLE_HTML),
        ("custom_element_html", CUSTOM_ELEMENT_HTML),
        ("entity_heavy_html", ENTITY_HEAVY_HTML),
        ("table_html", TABLE_HTML),
        ("hacker_news_fixture", HACKER_NEWS_FIXTURE),
        ("wikipedia_small_fixture", WIKIPEDIA_SMALL_FIXTURE),
    ];

    let strategies = [
        ("auto", options_for(TierStrategy::Auto)),
        ("tier2", options_for(TierStrategy::Tier2)),
    ];

    let mut group = c.benchmark_group("convert");
    for (case_name, html) in cases {
        group.throughput(Throughput::Bytes(html.len() as u64));

        for (strategy_name, options) in &strategies {
            group.bench_with_input(
                BenchmarkId::new(*strategy_name, case_name),
                &(html, options),
                |b, (html, options)| {
                    b.iter(|| {
                        let result =
                            convert(black_box(*html), Some((*options).clone())).expect("convert");
                        black_box(result);
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_convert_cases);
criterion_main!(benches);
