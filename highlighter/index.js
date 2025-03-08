import { createHighlighter } from 'shiki'
import { readFileSync } from 'fs'
import readline from "readline/promises"
import {
    transformerNotationDiff,
    transformerNotationHighlight,
    transformerNotationWordHighlight,
    transformerNotationErrorLevel,
    transformerMetaHighlight,
    transformerMetaWordHighlight,
    transformerNotationFocus
} from "@shikijs/transformers"

const theme = JSON.parse(readFileSync('hanekawa.json', 'utf-8'))

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
})

const highlighter = createHighlighter({
    langs: ["javascript", "rust", "c#", "c", "asm", "sh", "ts"],
    themes: [theme]
})

const tokenMap = {
    fn: "meta.declaration.annotation"
}

let loadedLangs;
let sum = 0

function time(start) {
    let time = performance.now() - start
    sum += time
    // console.error(sum)
}

rl.on('line', async (line) => {
    let start = performance.now();
    const data = JSON.parse(line)
    let { lang, inline, code, meta } = data
    if(lang[0] == ".") {
        let scope = lang.slice(1)
        let color
        if(tokenMap[scope]) scope = tokenMap[scope]
        for(let tokens of theme.tokenColors) {
            if(tokens.scope?.includes(scope)) {
                color = tokens.settings.foreground
                break
            }
        }
        let html
        // shiki does this for us, but we need to do it manually here
        code = simplehtmlentities(code)
        if(color) {
            html = `<code data-pretty-code-figure style="color: ${color}">${code}</code>`
        } else {
            html = code
        }
        time(start)
        console.log(JSON.stringify({ html, elapsed: performance.now() - start, sum }))
        return
    }
    let shiki = await highlighter
    if(!loadedLangs) loadedLangs = shiki.getLoadedLanguages()
    if(!loadedLangs.includes(lang)) {
        await shiki.loadLanguage(lang)
        loadedLangs = shiki.getLoadedLanguages()
    }
    let html = shiki.codeToHtml(code, {
        lang,
        structure: inline ? 'inline' : 'classic',
        theme: "Hanekawa",
        meta: {
            "data-pretty-code": "",
            __raw: meta
        },
        transformers: [
            transformerNotationDiff(),
            transformerNotationHighlight(),
            transformerNotationWordHighlight(),
            transformerNotationErrorLevel(),
            transformerNotationFocus()
        ]
    })
    if(inline) html = `<code data-pretty-code-figure>${html}</code>`
    time(start)
    let out = JSON.stringify({ html, elapsed: performance.now() - start, sum });
    console.log(out)
})

function simplehtmlentities(str) {
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
}