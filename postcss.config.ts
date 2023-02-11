import { ConfigFn } from "postcss-load-config";
import postcssPresetEnv from "postcss-preset-env";
import postcssMixins from "postcss-mixins";

const config: ConfigFn = () => ({
    plugins: [postcssMixins(), postcssPresetEnv({ stage: 1 })],
});

export default config;
