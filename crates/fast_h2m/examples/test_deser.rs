#![allow(missing_docs)]
use fast_h2m::ConversionOptions;

fn main() {
    let json = r#"{"headingStyle":"","listIndentType":"","listIndentWidth":2,"bullets":"-*+","strongEmSymbol":"*","escapeAsterisks":false,"escapeUnderscores":false,"escapeMisc":false,"escapeAscii":false,"codeLanguage":"","autolinks":true,"defaultTitle":false,"brInTables":false,"highlightStyle":"","extractMetadata":true,"whitespaceMode":"","stripNewlines":false,"wrap":false,"wrapWidth":80,"convertAsInline":false,"subSymbol":"","supSymbol":"","newlineStyle":"spaces","codeBlockStyle":"tildes","keepInlineImagesIn":null,"preprocessing":{"enabled":false,"preset":"","removeNavigation":false,"removeForms":false},"encoding":"utf-8","debug":false,"stripTags":null,"preserveTags":null,"skipImages":false,"linkStyle":"","outputFormat":"","includeDocumentStructure":false,"extractImages":false,"maxImageSize":5242880,"captureSvg":false,"inferDimensions":true}"#;

    let opts: ConversionOptions = serde_json::from_str(json).unwrap();
    println!("code_block_style: {:?}", opts.code_block_style);

    let result =
        fast_h2m::convert("<pre><code>some code</code></pre>", Some(opts)).unwrap();
    println!("result: {:?}", result.content);
}
