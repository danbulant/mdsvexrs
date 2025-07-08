# MDSvexRs

Vite/sveltekit plugin that uses `mdsvexrs-wasm` to convert markdown to svelte files in an optimized way.
This plugin generates raw html (using `{@html}` tags) that result in faster build times (less js generated, less js for esbuild to process), faster changes (setting innerHtml is faster than using JSDOM) and slightly faster loading times.

Main use case was allowing ~2k LoC markdown with embedded svelte to build with less than 20GB of RAM (yes, that's how much esbuild used).

## Use

Install this package

```
pnpm install mdsvexrs
```

And add it to preprocess under `svelte.config.js`:

```js
import { mdsvexrs } from 'mdsvexrs'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: ...,
	extensions: ['.svelte', '.md'],

	preprocess: [
        mdsvexrs({ layout: "$lib/layout.svelte" })
	]
};
```

Note that a layout *is* required and requires a static path - use `$lib` prefix and put your layout under `src/lib`. Layout is a svelte file
that accepts route data and markdown frontmatter as inputs.

As a minimal layout, just render children:

```svelte
<!-- src/lib/layout.svelte -->
<slot />
```

Frontmatter is passed as props, and props passed to markdown component (such as sveltekit `data`) are also passed as props to layout.

Note that markdown scripts are assumed to be 'old' svelte and not runes. They use `$$props` to pass props to layout, which won't work in runes mode (i.e. if you use `$state` or similar in .md `<script>`).

## Added features

Inline code highlighting - use it either via appending `{:lang}` inside inline code or by setting `defaultLang` in frontmatter.

## Differences from MDSvex

- Layout is required
- Most svelte syntax is not supported - this library uses HTML oriented markdown parser, which is then passed unescaped to svelte.
  - some easy common fixes are simply quoting argument values if they contain spaces, or moving template logic into separate components and just referencing them
- custom html tags need to be enumerated in config (`customTags: ['a']` in `mdsvexrs({ ... })`). They are still imported from layout.
  - they are uppercased during import and used as such, so the above will result in `<A href...></A>`.
  - note that overusing custom tags does come with a performance penalty, especially with very common tags like `p` or `code`.
- multiple layouts are not supported