import {
    defineConfig,
    presetWind,
    presetIcons,
    presetAttributify,
    Extractor,
} from "unocss";

const re_class = /class\s*:\s*"([\w -:]+)"/g;
const re_attributify = /u_([\w_]+)\s*:\s*"([\w -:~]+)"/g;

function extractorDioxus(): Extractor {
    return {
        name: "dioxus",
        extract({ code }) {
            let result = new Set<string>();
            for (const match of code.matchAll(re_class)) {
                result.add(match[1]);
            }
            for (const match of code.matchAll(re_attributify)) {
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
            prefix: "u-",
            strict: true,
        }),
    ],
    extractors: [extractorDioxus()],
    theme: {
        extend: {
            container: {
                center: true,
            },
        },
    },
});
