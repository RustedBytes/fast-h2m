import pytest

import fast_h2m


def test_convert_returns_result_dict():
    result = fast_h2m.convert("<h1>Hello</h1><p>World</p>")

    assert isinstance(result, dict)
    assert "Hello" in result["content"]
    assert "World" in result["content"]
    assert result["warnings"] == []


def test_convert_to_markdown_returns_content():
    markdown = fast_h2m.convert_to_markdown("<h1>Hello</h1><p>World</p>")

    assert isinstance(markdown, str)
    assert "Hello" in markdown
    assert "World" in markdown


def test_options_dict_changes_behavior():
    html = "<p>one two three four five six seven eight nine ten</p>"
    markdown = fast_h2m.convert_to_markdown(
        html,
        {"wrap": True, "wrap_width": 12},
    )

    assert "\n" in markdown.strip()


def test_invalid_options_raise_value_error():
    with pytest.raises(ValueError):
        fast_h2m.convert("<p>Hello</p>", {"not_a_real_option": True})


def test_metadata_is_included_by_default():
    result = fast_h2m.convert(
        "<html><head><title>Page title</title></head><body><p>Body</p></body></html>"
    )

    assert result["metadata"]["document"]["title"] == "Page title"


def test_fast_dom_tier_strategy_is_available():
    html = """
    <article>
      <h1>Hello</h1>
      <p>A <strong>fast</strong> path with <a href="https://example.com">a link</a>.</p>
      <script>ignored()</script>
    </article>
    """

    markdown = fast_h2m.convert_to_markdown(html, {"tier_strategy": "fast_dom"})

    assert "# Hello" in markdown
    assert "**fast**" in markdown
    assert "[a link](https://example.com)" in markdown
    assert "ignored" not in markdown


def test_fast_dom_tier_strategy_accepts_camel_case_alias():
    markdown = fast_h2m.convert_to_markdown(
        "<h1>Hello</h1><p>World</p>",
        {"tierStrategy": "fast_dom"},
    )

    assert "# Hello" in markdown
    assert "World" in markdown
