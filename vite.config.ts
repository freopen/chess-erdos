import { defineConfig } from "vite";
import { createHtmlPlugin } from "vite-plugin-html";

export default defineConfig({
    plugins: [
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
                target: "http://127.0.0.1:3001",
                changeOrigin: true,
            },
        },
        // watch: {
        //     atomic: true,
        // },
        hmr: {
            port: 3000,
        },
    },
    clearScreen: false,
});
