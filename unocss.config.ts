import {
    defineConfig,
    presetWind,
    presetIcons,
    presetAttributify,
    Extractor,
} from "unocss";

const re = /u_(\w+)\s*:\s*"([\w -]+)"/g;

function extractorDioxus(): Extractor {
    return {
        name: "dioxus",
        extract({ code }) {
            let result = new Set<string>();
            for (const match of code.matchAll(re)) {
                result.add(`[u-${match[1].replace("_", "-")}~="${match[2]}"]`);
            }
            return result;
        },
    };
}

export default defineConfig({
    presets: [
        presetWind(),
        presetIcons(),
        presetAttributify({
            strict: true,
            prefix: "u-",
        }),
    ],
    extractors: [extractorDioxus()],
});
