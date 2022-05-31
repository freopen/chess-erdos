import { defineConfig } from "vite";
import { createHtmlPlugin } from "vite-plugin-html";
import Icons from "unplugin-icons/vite";

export default defineConfig({
    plugins: [
        createHtmlPlugin({
            template: "src/client/index.html",
            minify: true,
        }),
        Icons({
            compiler: "raw",
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
