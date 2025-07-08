use std::{
    sync::LazyLock,
    time::{Duration/*, Instant*/},
};

use itertools::Itertools;
use markdown::{
    mdast::{
        Blockquote, Break, Code, Definition, Delete, Emphasis, FootnoteDefinition,
        FootnoteReference, Heading, Html, Image, ImageReference, InlineCode, InlineMath, Link,
        LinkReference, List, ListItem, Math, MdxFlowExpression, MdxJsxFlowElement,
        MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node, Paragraph, Root, Strong, Table,
        TableCell, TableRow, Text, ThematicBreak, Toml, Yaml,
    },
    unist::Position,
    Constructs,
};
use serde::Serialize;
use serde_json::Value;
use syntect::{
    dumps::from_uncompressed_data, easy::HighlightLines, highlighting::ThemeSet, html::{append_highlighted_html_for_styled_line, IncludeBackground}, parsing::{SyntaxDefinition, SyntaxSet}, util::LinesWithEndings
};

#[derive(Debug)]
struct ToHtmlResult {
    html: String,
    svelte: bool,
}

impl ToHtmlResult {
    fn new(html: String, svelte: bool) -> Self {
        Self { html, svelte }
    }

    fn empty() -> Self {
        Self::new("".to_string(), false)
    }
}

trait ToHtml {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult;

    fn visit(&self, _ctx: &mut Context) {}
}

impl<T> ToHtml for Vec<T>
where
    T: ToHtml,
{
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        merge(&self.iter().map(|t| t.to_html(ctx)).collect::<Vec<_>>())
    }

    fn visit(&self, ctx: &mut Context) {
        for t in self {
            t.visit(ctx);
        }
    }
}

impl ToHtml for Root {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        self.children.to_html(ctx)
    }

    fn visit(&self, ctx: &mut Context) {
        self.children.visit(ctx);
    }
}

impl ToHtml for Blockquote {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(
            format!("<blockquote>{}</blockquote>", children.html),
            children.svelte,
        )
    }
}

impl ToHtml for FootnoteDefinition {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }

    fn visit(&self, _ctx: &mut Context) {
        todo!()
    }
}

impl ToHtml for MdxJsxFlowElement {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

impl ToHtml for List {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        // todo!()
        let litype = match self.ordered {
            true => "ol",
            false => "ul",
        };
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(
            format!("<{}>{}</{}>", litype, children.html, litype),
            children.svelte,
        )
    }
}

impl ToHtml for MdxjsEsm {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

impl ToHtml for Toml {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

impl ToHtml for Yaml {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        ToHtmlResult::empty()
    }

    fn visit(&self, ctx: &mut Context) {
        let value = serde_yaml::from_str(&self.value).unwrap();
        ctx.yaml = Some(value);
    }
}

impl ToHtml for Break {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        ToHtmlResult::new("<br>".to_string(), false)
    }
}

static LANG_HINT_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\{:(?<lang>[\w.]+)\}$").unwrap());

impl ToHtml for InlineCode {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let value = &self.value;
        // if value ends with {lang} then it's a language hint
        let output = if let Some(caps) = LANG_HINT_REGEX.captures(value) {
            let lang = &caps["lang"];
            let code = &value[..value.len() - lang.len() - 3];
            ctx.highlight(HighlightRequest {
                lang: lang.to_string(),
                inline: true,
                code: code.to_string(),
                meta: None,
            })
        } else if let Some(lang) = &ctx.default_lang {
            ctx.highlight(HighlightRequest {
                lang: lang.clone(),
                inline: true,
                code: value.clone(),
                meta: None,
            })
        } else {
            format!("<code>{}</code>", html_encode(&self.value))
        };
        ToHtmlResult::new(output, false)
    }
}

impl ToHtml for InlineMath {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        ToHtmlResult::new(self.value.clone(), false)
    }
}

impl ToHtml for Delete {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for Emphasis {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<em>{}</em>", children.html), children.svelte)
    }
}

impl ToHtml for MdxTextExpression {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

impl ToHtml for FootnoteReference {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for Html {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let value = self.value.clone();
        if value.starts_with("<script") {
            ctx.script = Some(value);

            return ToHtmlResult::empty();
        }
        ToHtmlResult::new(value, true)
    }
}

impl ToHtml for Image {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        let alt = &self.alt;
        let title = self
            .title
            .as_ref()
            .map(|t| format!(" title=\"{}\"", t))
            .unwrap_or_default();
        let url = &self.url;
        ToHtmlResult::new(
            format!("<img src=\"{}\" alt=\"{}\"{}>", url, alt, title),
            false,
        )
    }
}

impl ToHtml for ImageReference {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for MdxJsxTextElement {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

impl ToHtml for Link {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        let title = self
            .title
            .as_ref()
            .map(|t| format!(" title=\"{}\"", t))
            .unwrap_or_default();
        ToHtmlResult::new(
            format!("<a href=\"{}\"{}>{}</a>", self.url, title, children.html),
            children.svelte,
        )
    }
}

impl ToHtml for LinkReference {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for Strong {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(
            format!("<strong>{}</strong>", children.html),
            children.svelte,
        )
    }
}

impl ToHtml for Text {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        ToHtmlResult::new(html_encode(&self.value), false)
    }
}

impl ToHtml for Code {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let value = &self.value;
        let lang = self.lang.as_ref().or(ctx.default_lang.as_ref());
        let highlighted = if let Some(lang) = lang {
            ctx.highlight(HighlightRequest {
                code: value.clone(),
                inline: false,
                lang: lang.clone(),
                meta: self.meta.clone(),
            })
        } else {
            format!("<pre><code>{}</code></pre>", html_encode(&self.value))
        };
        ToHtmlResult::new(highlighted, false)
    }
}

impl ToHtml for Math {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for MdxFlowExpression {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        unimplemented!()
    }
}

fn slug(str: &str) -> String {
    str.to_lowercase().replace(" ", "-")
}

impl ToHtml for Heading {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        let text = self
            .children
            .iter()
            .filter_map(|c| match c {
                Node::Text(t) => Some(t.value.clone()),
                _ => None,
            })
            .join("");
        let mut slug = slug(&text);
        if ctx.titles.iter().any(|t| t.id == slug) {
            let mut i = 1;
            while ctx.titles.iter().any(|t| t.id == format!("{}-{}", slug, i)) {
                i += 1;
            }
            slug = format!("{}-{}", slug, i);
        }
        ctx.titles.push(Title {
            level: self.depth,
            text: text.clone(),
            id: slug.clone(),
            pos: self.position.clone(),
        });
        ToHtmlResult::new(
            format!(
                "\n<h{} id=\"{}\">{}</h{}>\n",
                self.depth, slug, children.html, self.depth
            ),
            children.svelte,
        )
    }
}

impl ToHtml for Table {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<table>{}</table>", children.html), children.svelte)
    }
}

impl ToHtml for ThematicBreak {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        ToHtmlResult::new("\n<hr>\n".to_string(), false)
    }
}

impl ToHtml for TableRow {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<tr>{}</tr>", children.html), children.svelte)
    }
}

impl ToHtml for TableCell {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<td>{}</td>", children.html), children.svelte)
    }
}

impl ToHtml for ListItem {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<li>{}</li>", children.html), children.svelte)
    }
}

impl ToHtml for Definition {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
    }
}

impl ToHtml for Paragraph {
    fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<p>{}</p>", children.html), children.svelte)
    }
}

macro_rules! node_impl {
    ($($node:ident($name:ident)),+) => {
        impl ToHtml for Node {
            fn to_html(&self, ctx: &mut Context) -> ToHtmlResult {
                match self {
                    $(markdown::mdast::Node::$node($name) => $name.to_html(ctx)),+
                }
            }
            fn visit(&self, ctx: &mut Context) {
                match self {
                    $(markdown::mdast::Node::$node($name) => $name.visit(ctx)),+
                }
            }
        }
    }
}

node_impl!(
    Root(root),
    Blockquote(blockquote),
    FootnoteDefinition(footnote_definition),
    MdxJsxFlowElement(mdx_jsx_flow_element),
    List(list),
    MdxjsEsm(mdxjs_esm),
    Toml(toml),
    Yaml(yaml),
    Break(rbreak),
    InlineCode(inline_code),
    InlineMath(inline_math),
    Delete(delete),
    Emphasis(emphasis),
    MdxTextExpression(mdx_text_expression),
    FootnoteReference(footnote_reference),
    Html(html),
    Image(image),
    ImageReference(image_reference),
    MdxJsxTextElement(mdx_jsx_text_element),
    Link(link),
    LinkReference(link_reference),
    Strong(strong),
    Text(text),
    Code(code),
    Math(math),
    MdxFlowExpression(mdx_flow_expression),
    Heading(heading),
    Table(table),
    ThematicBreak(thematic_break),
    TableRow(table_row),
    TableCell(table_cell),
    ListItem(list_item),
    Definition(definition),
    Paragraph(paragraph)
);

fn svelte_html_encode(string: String) -> String {
    String::from("{@html `") + &string.replace("\\", "\\\\").replace("`", "\\`") + "`}"
}

fn merge(results: &[ToHtmlResult]) -> ToHtmlResult {
    let chunked = results.iter().chunk_by(|r| r.svelte);
    let mut html = chunked.into_iter().map(|(svelte, results)| {
        let html = results.map(|r| r.html.clone()).join("");
        ToHtmlResult::new(html, svelte)
    });
    let Some(first) = html.next() else {
        return ToHtmlResult::new("".to_string(), false);
    };
    if let Some(second) = html.next() {
        ToHtmlResult::new(
            [first, second]
                .into_iter()
                .chain(html)
                .map(|r| {
                    if r.html.is_empty() {
                        return "".to_string();
                    }
                    if r.svelte {
                        r.html
                    } else {
                        svelte_html_encode(r.html)
                    }
                })
                .join(""),
            true,
        )
    } else {
        first
    }
}

// from markdown-rs
pub fn html_encode(value: &str) -> String {
    let encode_html = true; // originally a param
                            // It’ll grow a bit bigger for each dangerous character.
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut start = 0;

    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\0') || (encode_html && matches!(byte, b'&' | b'"' | b'<' | b'>')) {
            result.push_str(&value[start..index]);
            result.push_str(match byte {
                b'\0' => "�",
                b'&' => "&amp;",
                b'"' => "&quot;",
                b'<' => "&lt;",
                // `b'>'`
                _ => "&gt;",
            });

            start = index + 1;
        }

        index += 1;
    }

    result.push_str(&value[start..]);

    result
}

fn finish(res: ToHtmlResult) -> String {
    if res.svelte {
        res.html
    } else {
        svelte_html_encode(res.html)
    }
}

#[derive(Serialize)]
pub struct Title {
    pub level: u8,
    pub text: String,
    pub id: String,
    pub pos: Option<Position>,
}

pub struct Context {
    pub yaml: Option<serde_json::Map<String, Value>>,
    pub default_lang: Option<String>,
    pub script: Option<String>,
    pub options: MdsvexrsOptions,
    pub titles: Vec<Title>,

    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,

    // pub(crate) highlight_times: Duration,
    // pub(crate) parse_time: Duration,
    // pub(crate) visit_time: Duration,
    // pub(crate) convert_time: Duration,
}

#[derive(Serialize)]
struct HighlightRequest {
    code: String,
    inline: bool,
    lang: String,
    meta: Option<String>,
}

pub struct MdsvexrsOptions {
    pub layout: String,
    // pub path: String,
}

impl Context {
    pub fn new(options: MdsvexrsOptions) -> Self {

        let syntax_set: SyntaxSet = from_uncompressed_data(include_bytes!("../target/syntax_cache.packdump")).unwrap();
        let theme_set = ThemeSet::load_defaults();

        Context {
            // child,
            // bufread,
            syntax_set,
            theme_set,
            yaml: None,
            titles: Vec::new(),
            default_lang: None,
            script: None,
            options,
            // highlight_times: Duration::ZERO,
            // parse_time: Duration::ZERO,
            // visit_time: Duration::ZERO,
            // convert_time: Duration::ZERO,
        }
    }

    fn highlight(&mut self, code: HighlightRequest) -> String {
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        // #[cfg(not(target_arch = "wasm32"))]
        // let start = Instant::now();

        let mut lang = &code.lang;
        if lang.is_empty() {
            if let Some(default) = &self.default_lang {
                lang = default;
            } else {
                return format!("<pre><code>{}</code></pre>", html_encode(&code.code));
            }
        }
        let syntax = self.syntax_set.find_syntax_by_token(lang);
        let syntax = match syntax {
            Some(t) => t,
            None => {
                return format!("<pre><code lang=\"{}\">{}</code></pre>", html_encode(lang), html_encode(&code.code));
            }
        };
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut string = String::new();

        match code.inline {
            true => {
                let regions = highlighter
                    .highlight_line(&code.code, &self.syntax_set)
                    .unwrap();
                string += &format!("<code lang=\"{}\">", html_encode(lang));
                append_highlighted_html_for_styled_line(
                    &regions[..],
                    IncludeBackground::No,
                    &mut string,
                )
                .unwrap();
                string += "</code>";
            }
            false => {
                string += &format!("<pre><code lang=\"{}\">", html_encode(lang));
                for line in LinesWithEndings::from(&code.code) {
                    let regions = highlighter.highlight_line(line, &self.syntax_set).unwrap();
                    append_highlighted_html_for_styled_line(
                        &regions[..],
                        IncludeBackground::No,
                        &mut string,
                    )
                    .unwrap();
                }
                string += "</code></pre>\n";
            }
        };

        // #[cfg(not(target_arch = "wasm32"))] {
        //     self.highlight_times += start.elapsed();
        // }
        string
    }

    fn resolve_layout(&self) -> &str {
        &self.options.layout
    }

    pub fn convert(&mut self, input: &str) -> String {
        // #[cfg(not(target_arch = "wasm32"))]
        // let start = Instant::now();
        let ast = markdown::to_mdast(input, &DEFAULT_MD_OPTIONS).unwrap();
        // #[cfg(not(target_arch = "wasm32"))] {
        //     self.parse_time = start.elapsed();
        // }
        // #[cfg(not(target_arch = "wasm32"))]
        // let start = Instant::now();
        ast.visit(self);
        // #[cfg(not(target_arch = "wasm32"))] {
        //     self.visit_time = start.elapsed();
        // }

        if let Some(yaml) = &self.yaml {
            if let Some(val) = yaml.get("defaultLang") {
                self.default_lang = Some(val.as_str().unwrap().to_string());
            }
        }

        // #[cfg(not(target_arch = "wasm32"))]
        // let start = Instant::now();
        let res = ast.to_html(self);
        let html = finish(res);
        // #[cfg(not(target_arch = "wasm32"))] {
        //     self.convert_time = start.elapsed();
        // }

        if let Some(yaml) = &mut self.yaml {
            yaml.insert(
                "titles".to_string(),
                serde_json::to_value(&self.titles).unwrap(),
            );
        }

        let value = self
            .script
            .clone()
            .unwrap_or_else(|| String::from("<script></script>"));

        let script = {
            let end = value.find('>').expect(
                "Unclosed script tag (found <script but not >). May be a bug with Markdown parser.",
            ) + 1;
            let mut script = value[..end].to_string();
            let layout = self.resolve_layout();
            script += format!("import MDXLayout from \"{}\";", layout).as_str();
            script += &value[end..];
            script
        };

        let frontmatter =
            (|| serde_json::to_string(self.yaml.as_ref()?).ok())().unwrap_or("{}".to_string());

        format!(
            "<script context=\"module\">export const metadata = {frontmatter}</script>
{script}
<MDXLayout {{...metadata}} {{...$$restProps}}>
{html}
</MDXLayout>"
        )
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // pub fn print_timings(&self) {
    //     println!("Parse: {:?}", self.parse_time);
    //     println!("Visit: {:?}", self.visit_time);
    //     println!("Convert: {:?}", self.convert_time);
    //     println!("Highlight: {:?}", self.highlight_times);
    // }
    // #[cfg(target_arch = "wasm32")]
    pub fn print_timings(&self) {
        println!("wasm timings are not available");
    }
}

pub(crate) const DEFAULT_MD_OPTIONS: markdown::ParseOptions = markdown::ParseOptions {
    constructs: Constructs {
        attention: true,
        autolink: true,
        block_quote: true,
        character_escape: true,
        character_reference: true,
        code_indented: true,
        code_fenced: true,
        code_text: true,
        definition: true,
        frontmatter: true,
        gfm_autolink_literal: true,
        gfm_footnote_definition: true,
        gfm_label_start_footnote: true,
        gfm_strikethrough: true,
        gfm_table: true,
        gfm_task_list_item: true,
        hard_break_escape: true,
        hard_break_trailing: true,
        heading_atx: true,
        heading_setext: true,
        html_flow: true,
        html_text: true,
        label_start_image: true,
        label_start_link: true,
        label_end: true,
        list_item: true,
        math_flow: true,
        math_text: true,
        mdx_esm: false,
        mdx_expression_flow: false,
        mdx_expression_text: false,
        mdx_jsx_flow: false,
        mdx_jsx_text: false,
        thematic_break: true,
    },
    math_text_single_dollar: true,
    gfm_strikethrough_single_tilde: true,
    mdx_expression_parse: None,
    mdx_esm_parse: None,
};
