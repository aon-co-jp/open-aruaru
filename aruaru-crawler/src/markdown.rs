use pulldown_cmark::{html, Options, Parser};

pub fn markdown_to_safe_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut raw_html = String::new();
    html::push_html(&mut raw_html, parser);

    ammonia::Builder::default()
        .add_tags(["details", "summary", "kbd", "mark"])
        .add_generic_attributes(["class", "id", "title"])
        .clean(&raw_html)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::markdown_to_safe_html;

    #[test]
    fn removes_script() {
        let html = markdown_to_safe_html("# Title\n<script>alert(1)</script>");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(!html.contains("<script>"));
    }
}
