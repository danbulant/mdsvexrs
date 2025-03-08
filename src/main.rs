use std::{io::{stdin, BufRead, BufReader, Read, Write}, process::{Child, ChildStdout, Command, Stdio}, sync::LazyLock, time::{Duration, Instant}};

use itertools::Itertools;
use markdown::{mdast::{Blockquote, Break, Code, Definition, Delete, Emphasis, FootnoteDefinition, FootnoteReference, Heading, Html, Image, ImageReference, InlineCode, InlineMath, Link, LinkReference, List, ListItem, Math, MdxFlowExpression, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node, Paragraph, Root, Strong, Table, TableCell, TableRow, Text, ThematicBreak, Toml, Yaml}, unist::Position, Constructs};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use clap::Parser;

#[derive(Debug)]
struct ToHtmlResult {
    html: String,
    svelte: bool
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

impl<T> ToHtml for Vec<T> where T: ToHtml {
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
        ToHtmlResult::new(format!("<blockquote>{}</blockquote>", children.html), children.svelte)
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
            false => "ul"
        };
        let children = self.children.to_html(ctx);
        ToHtmlResult::new(format!("<{}>{}</{}>", litype, children.html, litype), children.svelte)
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

static LANG_HINT_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\{:(?<lang>[\w.]+)\}$").unwrap());

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
                meta: None
            })
        } else if let Some(lang) = &ctx.default_lang {
            ctx.highlight(HighlightRequest {
                lang: lang.clone(),
                inline: true,
                code: value.clone(),
                meta: None
            })
        } else {
            format!("<code>{}</code>", html_encode(&self.value))
        };
        ToHtmlResult::new(output, false)
    }
}

impl ToHtml for InlineMath {
    fn to_html(&self, _ctx: &mut Context) -> ToHtmlResult {
        todo!()
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
        let title = self.title.as_ref().map(|t| format!(" title=\"{}\"", t)).unwrap_or_default();
        let url = &self.url;
        ToHtmlResult::new(format!("<img src=\"{}\" alt=\"{}\"{}>", url, alt, title), false)
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
        let title = self.title.as_ref().map(|t| format!(" title=\"{}\"", t)).unwrap_or_default();
        ToHtmlResult::new(format!("<a href=\"{}\"{}>{}</a>", self.url, title, children.html), children.svelte)
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
        ToHtmlResult::new(format!("<strong>{}</strong>", children.html), children.svelte)
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
                meta: self.meta.clone()
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
        let text = self.children.iter().filter_map(|c| {
            match c {
                Node::Text(t) => Some(t.value.clone()),
                _ => None
            }
        }).join("");
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
            pos: self.position.clone()
        });
        ToHtmlResult::new(format!("\n<h{} id=\"{}\">{}</h{}>\n", self.depth, slug, children.html, self.depth), children.svelte)
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
            [first, second].into_iter().chain(html).map(|r| {
                if r.html.is_empty() {
                    return "".to_string();
                }
                if r.svelte {
                    r.html
                } else {
                    svelte_html_encode(r.html)
                }
            }).join(""),
            true
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
struct Title {
    level: u8,
    text: String,
    id: String,
    pos: Option<Position>
}

struct Context {
    child: Child,
    bufread: BufReader<ChildStdout>,
    yaml: Option<serde_json::Map<String, Value>>,
    default_lang: Option<String>,
    script: Option<String>,
    args: Args,
    titles: Vec<Title>,

    highlight_times: Duration,
    js_times: Duration,
    js_sum: f64
}

#[derive(Serialize)]
struct HighlightRequest {
    code: String,
    inline: bool,
    lang: String,
    meta: Option<String>
}

#[derive(Deserialize)]
struct HighlightResponse {
    html: String,

    elapsed: f64,
    sum: f64
}

impl Context {
    fn new(args: Args) -> Self {
        let mut child = Command::new("node")
            .arg(".")
            .current_dir("highlighter")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let bufread = BufReader::new(stdout);

        Context { child, bufread, yaml: None, titles: Vec::new(), default_lang: None, script: None, args, highlight_times: Duration::ZERO, js_times: Duration::ZERO, js_sum: 0. }
    }

    fn highlight(&mut self, code: HighlightRequest) -> String {
        let start = Instant::now();
        
        let stdin = self.child.stdin.as_mut().unwrap();
        let data = serde_json::to_string(&code).unwrap() + "\n";
        stdin.write_all(data.as_bytes()).unwrap();
        
        let mut buf = String::new();
        let _line = self.bufread.read_line(&mut buf).unwrap();
        let res: HighlightResponse = serde_json::from_str(&buf).unwrap();

        self.highlight_times += start.elapsed();
        self.js_times += Duration::from_nanos((res.elapsed * 1_000_000.) as u64);
        self.js_sum = res.sum;
        res.html
    }

    fn resolve_layout(&self) -> &str {
        &self.args.layout
        // Path::new(&self.args.layout).
    }
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    layout: String,
    #[arg(short, long)]
    path: String,
    #[arg(long)]
    timings: bool
}

/// Converts markdown to svelte code, MDSvex alternative. Expects trusted code!
fn main() {
    let options = markdown::ParseOptions {
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
        ..Default::default()
    };
    let args = Args::parse();
    let mut ctx = Context::new(args);
    let mut input = String::new();

    
    let start = Instant::now();
    stdin().read_to_string(&mut input).unwrap();
    let stdin_read = start.elapsed();
    
    let start = Instant::now();
    let ast = markdown::to_mdast(&input, &options).unwrap();
    let ast_parse = start.elapsed();
    
    let start = Instant::now();
    ast.visit(&mut ctx);
    let ast_visit = start.elapsed();

    if let Some(yaml) = &ctx.yaml {
        if let Some(val) = yaml.get("defaultLang") {
            ctx.default_lang = Some(val.as_str().unwrap().to_string());
        }
    }
    
    let start = Instant::now();
    let res = ast.to_html(&mut ctx);
    let html = finish(res);
    let html_convert = start.elapsed();

    if ctx.args.timings {
        dbg!(stdin_read, ast_parse, ast_visit, html_convert, ctx.highlight_times, ctx.js_times, ctx.js_sum);
        return;
    }

    if let Some(yaml) = &mut ctx.yaml {
        yaml.insert("titles".to_string(), serde_json::to_value(&ctx.titles).unwrap());
    }
    
    let value = ctx.script.clone().unwrap_or_else(|| String::from("<script></script>"));

    let script = {
        let end = value.find('>').expect("Unclosed script tag (found <script but not >). May be a bug with Markdown parser.") + 1;
        let mut script = value[..end].to_string();
        let layout = ctx.resolve_layout();
        script += format!("import MDXLayout from \"{}\";", layout).as_str();
        script += &value[end..];
        script
    };

    let frontmatter = (|| {
        serde_json::to_string(&ctx.yaml?).ok()
    })().unwrap_or("{}".to_string());
    println!("<script context=\"module\">export const metadata = {}</script>", frontmatter);

    println!("{script}");
    println!("<MDXLayout {{...metadata}}>");
    println!("{}", html);
    println!("</MDXLayout>");
}