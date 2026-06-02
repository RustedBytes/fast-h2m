//! Criterion benchmarks for representative `h2md::convert` workloads.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use h2md::{convert, ConversionOptions, TierStrategy};

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
    <tr><td>h2md</td><td>Rust</td><td>HTML to Markdown conversion</td></tr>
    <tr><td>criterion</td><td>Rust</td><td>Statistical benchmarking</td></tr>
    <tr><td>astral-tl</td><td>Rust</td><td>HTML parser</td></tr>
  </tbody>
</table>
"#;

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
        ("simple_html", SIMPLE_HTML),
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
