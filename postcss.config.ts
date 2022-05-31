// @ts-nocheck
import { ConfigFn } from "postcss-load-config";
import postcssImport from "postcss-import";
import postcssUrl from "postcss-url";
import postcssNested from "postcss-nested";
import autoprefixer from "autoprefixer";
import cssnanoPlugin from "cssnano";
import postcssImageInliner from "postcss-image-inliner";
import { lookupCollection } from "@iconify/json";
import { IconSet } from "@iconify/tools";
import assert from "assert";

const matcher = /icon\(\s*['"]*([^\s'"\(\)]*)['"]*\s*\))/;

async function getIcon(names: Set<string>): { [key: string]: string } {}

const config: ConfigFn = () => {
    return {
        plugins: [
            {
                postcssPlugin: "iconify-inliner",
                async Once(root) {
                    const filter =
                        /^(background(?:-image)?)|(content)|(cursor)/;
                    const icons = new Set([]);
                    root.walkDecls(filter, (decl) => {
                        const match = decl.value.match(matcher);
                        if (match !== null) {
                            icons.add(match[1]);
                        }
                    });
                },
            },
            // postcssImport(),
            // postcssNested(),
            // autoprefixer(),
            // postcssUrl({
            //     url(asset) {
            //         assert(false);
            //         return asset.url.toUpperCase();
            //         // const url_parts = url.split("/", 1);
            //         // const svg = icons[url_parts[0]].toSVG(url_parts[1]);
            //         // assert(svg !== null);
            //         // return encodeURI(svg.toMinifiedString());
            //     },
            // }),
            cssnanoPlugin({
                preset: ["default"],
            }),
        ],
    };
};

export default config;
