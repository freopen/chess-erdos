import { defineConfig } from "vite";
import { ViteRsw } from "vite-plugin-rsw";
import { createHtmlPlugin } from "vite-plugin-html";

export default defineConfig({
    plugins: [
        ViteRsw(),
        createHtmlPlugin({
            minify: true,
        }),
    ],
});
