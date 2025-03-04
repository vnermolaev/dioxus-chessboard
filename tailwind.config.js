module.exports = {
    mode: "all",
    content: [
        // include all rust, html and css files in the src directory
        "./**/*.{rs,html}",
        // include all html files in the output (dist) directory
        "./dist/**/*.html",
    ],
    theme: {
        extend: {},
    },
    plugins: [],
}