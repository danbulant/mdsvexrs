# MDSvexRS

A faster markdown preprocessor for svelte. Compiles `.md` files into `.svelte` with non-reactive blocks wrapped in direct HTML for faster compilation and rendering.

Note that this, like the original MDSvex, trusts it's input and doesn't escape HTML or script files.

This version is not yet tested and published.

Note that not all svelte syntax is supported yet. Notably, only HTML-like content is handled. If you get invalid syntax, try moving it into a component and just referencing that component. Templates, Ifs etc are not supported.

Not all languages may be highlighted as syntect doesn't include support for all languages. Sublime syntax is supported and can be added on `Context.syntax_set`, but WASM don't have a way to set it - make an issue to embed the language instead.
