import * as wasm from "mdsvexrs-wasm"

/**
 * @param {import("./").Options} options 
 * @returns {import("./").Plugin}
 */
export function mdsvexrs(options) {
    let opts = wasm.get_default_options()
    opts.layout = options.layout
    if (options.customTags) {
        options.customTags.forEach(tag => {
            opts.add_custom_tag(tag)
        })
    }
    return {
        name: 'mdsvexrs',
        markup: ({ content, filename }) => {
            if(!filename || !filename.endsWith('.md')) return

            const code = wasm.render(content, opts)

            return {
                code
            }
        }
    }
}