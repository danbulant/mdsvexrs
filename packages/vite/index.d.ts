
export interface Options {
    layout: string;
    customTags?: string[];
}

interface Plugin {
    name: string;
    markup: (opts: { content: string, filename: string }) => { code: string } | undefined;
}

export function mdsvexrs(options: Options): Plugin;