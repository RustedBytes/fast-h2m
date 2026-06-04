//! Criterion benchmarks for representative `fast_h2m::convert` workloads.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "testkit")]
use fast_h2m::testkit::{prescan, tier1};
use fast_h2m::{ConversionOptions, TierStrategy, convert};

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
    include_str!("../../../fixtures/test_documents/html/issues/gh-121-hacker-news.html");

const WIKIPEDIA_SMALL_FIXTURE: &str =
    include_str!("../../../fixtures/test_documents/html/wikipedia/small_html.html");

const ELON_MUSK_WIKI_FIXTURE: &str = include_str!("../../../fixtures/elon-musk.html");

#[cfg(feature = "testkit")]
const WIKIPEDIA_MEDIUM_FIXTURE: &str =
    include_str!("../../../fixtures/test_documents/html/wikipedia/medium_python.html");

const ESCAPE_FREE_TEXT: &str =
    "This paragraph has ordinary words and spaces with no markdown punctuation to escape.";

const ESCAPE_HEAVY_TEXT: &str =
    r##"Escape ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~ repeatedly."##;

const BINARY_LIKE_INPUT: &str = "\0\0\0\0\0\0\0\0\0\0\0\0\0binary";

fn options_for(strategy: TierStrategy) -> ConversionOptions {
    ConversionOptions {
        tier_strategy: strategy,
        ..ConversionOptions::default()
    }
}

fn bench_convert_cases(c: &mut Criterion) {
    let cases = [
        ("elon_musk_wiki", ELON_MUSK_WIKI_FIXTURE),
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
        ("fast_dom", options_for(TierStrategy::FastDom)),
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

fn bench_public_focused_cases(c: &mut Criterion) {
    let mut escape_group = c.benchmark_group("text_escape_via_convert");
    let escape_options = ConversionOptions {
        escape_ascii: true,
        ..ConversionOptions::default()
    };
    for (case_name, text) in [
        ("escape_free", ESCAPE_FREE_TEXT),
        ("escape_heavy", ESCAPE_HEAVY_TEXT),
    ] {
        escape_group.throughput(Throughput::Bytes(text.len() as u64));
        escape_group.bench_with_input(
            BenchmarkId::new("plain_text", case_name),
            &(text, &escape_options),
            |b, (text, options)| {
                b.iter(|| {
                    let result =
                        convert(black_box(*text), Some((*options).clone())).expect("convert");
                    black_box(result);
                });
            },
        );
    }
    escape_group.finish();

    c.bench_function("validation/binary_reject", |b| {
        b.iter(|| {
            let result = convert(black_box(BINARY_LIKE_INPUT), None);
            let _ = black_box(result);
        });
    });
}

fn make_escape_threshold_text(len: usize, marker: &str) -> String {
    match marker {
        "no_marker" => "a".repeat(len),
        "first_marker" => {
            if len == 0 {
                String::new()
            } else {
                let mut text = String::with_capacity(len);
                text.push('*');
                text.push_str(&"a".repeat(len - 1));
                text
            }
        }
        "late_marker" => {
            if len == 0 {
                String::new()
            } else {
                let mut text = "a".repeat(len);
                text.replace_range(len - 1..len, "*");
                text
            }
        }
        "many_markers" => (0..len)
            .map(|idx| if idx % 4 == 0 { '*' } else { 'a' })
            .collect(),
        _ => unreachable!("unknown escape threshold marker case"),
    }
}

fn bench_escape_threshold_cases(c: &mut Criterion) {
    let escape_options = ConversionOptions {
        escape_ascii: true,
        ..ConversionOptions::default()
    };
    let cases: Vec<_> = [0usize, 8, 24, 64, 128, 512, 4096]
        .into_iter()
        .flat_map(|len| {
            ["no_marker", "first_marker", "late_marker", "many_markers"]
                .into_iter()
                .map(move |marker| (len, marker, make_escape_threshold_text(len, marker)))
        })
        .collect();

    let mut group = c.benchmark_group("text_escape_thresholds_via_convert");
    for (len, marker, text) in &cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(*marker, len),
            &(text.as_str(), &escape_options),
            |b, (text, options)| {
                b.iter(|| {
                    let result =
                        convert(black_box(*text), Some((*options).clone())).expect("convert");
                    black_box(result);
                });
            },
        );
    }
    group.finish();
}

#[cfg(feature = "testkit")]
fn bench_internal_focused_cases(c: &mut Criterion) {
    let prescan_cases = [
        ("simple_html", SIMPLE_HTML),
        ("wikipedia_small", WIKIPEDIA_SMALL_FIXTURE),
        ("wikipedia_medium", WIKIPEDIA_MEDIUM_FIXTURE),
    ];
    let mut prescan_group = c.benchmark_group("prescan");
    for (case_name, html) in prescan_cases {
        prescan_group.throughput(Throughput::Bytes(html.len() as u64));
        prescan_group.bench_with_input(BenchmarkId::from_parameter(case_name), html, |b, html| {
            b.iter(|| {
                let result = prescan::run(black_box(html));
                black_box(result);
            });
        });
    }
    prescan_group.finish();

    let tier1_cases = [
        ("simple_html", SIMPLE_HTML),
        ("table_html", TABLE_HTML),
        ("entity_heavy_html", ENTITY_HEAVY_HTML),
        ("wikipedia_small", WIKIPEDIA_SMALL_FIXTURE),
    ];
    let options = ConversionOptions {
        tier_strategy: TierStrategy::Tier1,
        ..ConversionOptions::default()
    };
    let mut tier1_group = c.benchmark_group("tier1_scanner");
    for (case_name, html) in tier1_cases {
        let (cleaned, report) = prescan::run(html);
        tier1_group.throughput(Throughput::Bytes(html.len() as u64));
        tier1_group.bench_with_input(
            BenchmarkId::from_parameter(case_name),
            &(cleaned.into_owned(), report.clone(), options.clone()),
            |b, (cleaned, report, options)| {
                b.iter(|| {
                    let result = tier1::run(
                        black_box(cleaned.as_str()),
                        black_box(report),
                        black_box(options),
                    );
                    let _ = black_box(result);
                });
            },
        );
    }
    tier1_group.finish();
}

#[cfg(feature = "testkit")]
criterion_group!(
    benches,
    bench_convert_cases,
    bench_public_focused_cases,
    bench_escape_threshold_cases,
    bench_internal_focused_cases
);
#[cfg(not(feature = "testkit"))]
criterion_group!(
    benches,
    bench_convert_cases,
    bench_public_focused_cases,
    bench_escape_threshold_cases
);
criterion_main!(benches);
