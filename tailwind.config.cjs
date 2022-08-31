module.exports = {
    mode: 'jit',
    content: [
        './src/**/*.{svelte,js,ts}',
    ],
    plugins: [require("@tailwindcss/typography"), require('daisyui')],

    // daisyUI config (optional)
    daisyui: {
        styled: true,
        themes: true,
        base: true,
        utils: true,
        logs: true,
        rtl: false,
        prefix: "",
        darkTheme: "dark",
    },
};
