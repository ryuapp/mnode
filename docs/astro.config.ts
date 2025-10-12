// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightThemeRapide from "starlight-theme-rapide";

// https://astro.build/config
export default defineConfig({
  integrations: [starlight({
    plugins: [starlightThemeRapide()],
    title: "mnode",
    social: [{
      icon: "github",
      label: "GitHub",
      href: "https://github.com/ryuapp/mnode",
    }],
    sidebar: [
      {
        label: "Guide",
        autogenerate: { directory: "guides" },
      },
    ],
  })],
  vite: {
    server: {
      fs: {
        allow: ["../"],
      },
    },
  },
});
