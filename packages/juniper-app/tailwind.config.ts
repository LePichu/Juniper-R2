import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{js,jsx,ts,tsx}", "./*.{html}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["IBM Plex Sans"],
        mono: ["IBM Plex Mono"],
        serif: ["IBM Plex Serif"],
      },
      colors: {
        "gray-primary": "#161616",
      },
    },
  },
  plugins: [],
};

export default config;
