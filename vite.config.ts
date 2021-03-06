import { defineConfig } from "vite";
import { ViteRsw } from "vite-plugin-rsw";
import { createHtmlPlugin } from "vite-plugin-html";

export default defineConfig({
    plugins: [
        // ViteRsw(),
        createHtmlPlugin({
            template: "html/index.html",
            minify: true,
        }),
    ],
    assetsInclude: ["generated/wasm/chess_erdos_bg.wasm"],
    build: {
        outDir: "generated/dist",
    },
    server: {
        proxy: {
            "/api": {
                target: "http://127.0.0.1:4000",
                changeOrigin: true,
            },
        },
        watch: {
            atomic: true,
        },
    },
    clearScreen: false,
});
