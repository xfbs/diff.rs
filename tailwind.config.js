module.exports = {
    content: ["**/*.{html,js,rs,css,rs}", "index.html"],
    corePlugins: {
        preflight: false,
    },
    plugins: [
        require('@tailwindcss/typography')
    ]
}
