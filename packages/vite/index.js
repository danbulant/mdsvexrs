import { render } from "mdsvexrs-wasm"

export function mdsvexrs(options) {
    return {
        name: 'mdsvexrs',
        markup: ({ content, filename }) => {
            if(!filename || !filename.endsWith('.md')) return

            return {
                code: render(content, options)
            }
        }
    }
}