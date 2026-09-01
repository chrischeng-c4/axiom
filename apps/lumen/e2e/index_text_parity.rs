use lumen::{text_index, tokenize, types::Analyzer};

#[test]
fn lumen_analyzers_keep_shared_index_text_token_parity() {
    let cases = [
        (
            "Hello, Durable World!",
            Analyzer::WhitespaceLower,
            text_index::Analyzer::WhitespaceLower,
        ),
        (
            "lumen 搜尋引擎",
            Analyzer::Jieba,
            text_index::Analyzer::Jieba,
        ),
        ("trace-id-123", Analyzer::Ngram, text_index::Analyzer::Ngram),
    ];

    for (text, lumen_analyzer, shared_analyzer) in cases {
        assert_eq!(
            tokenize::tokenize(text, lumen_analyzer),
            text_index::tokenize(text, shared_analyzer)
        );
    }
}
