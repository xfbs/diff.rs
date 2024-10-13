module.exports = {
    content: ["{src,static}/**/*.{html,js,rs,css,rs}", "index.html"],
    plugins: [
        require('@tailwindcss/typography')
    ],
    theme: {
        screens: {
            'xs': '400px',
            'sm': '640px',
            'md': '768px',
            'lg': '1024px',
            'xl': '1280px',
            '2xl': '1536px',
        }
    }
}
