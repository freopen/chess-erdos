import {
    defineConfig,
    presetWind,
    presetIcons,
    presetAttributify,
    Extractor,
} from "unocss";

const re_class = /"([\w -:]+)"/g;
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
                for (let value of match[2].split(" ")) {
                    result.add(`[u-${match[1].replace("_", "-")}~="${value}"]`);
                }
            }
            return result;
        },
    };
}

export default defineConfig({
    presets: [
        presetWind(),
        presetIcons({
            extraProperties: {
                display: "inline-block",
                "vertical-align": "middle",
            },
        }),
        presetAttributify({
            prefix: "u-",
            strict: true,
        }),
    ],
    extractors: [extractorDioxus()],
    theme: {},
});
