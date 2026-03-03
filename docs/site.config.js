export default {
  name: "null-e",
  description: "Developer disk cleanup CLI — find and remove build artifacts, caches, and dependencies",

  navLinks: [
    { label: "Docs", href: "#introduction" },
    { label: "Commands", href: "#commands" },
    { label: "Targets", href: "#targets" },
    { label: "GitHub", href: "https://github.com/us/null-e", external: true },
  ],

  sidebar: [
    {
      title: "Getting Started",
      children: [
        { title: "Introduction", slug: "introduction" },
        { title: "Installation", slug: "installation" },
        { title: "Quick Start", slug: "quick-start" },
      ],
    },
    {
      title: "Usage",
      children: [
        { title: "Commands", slug: "commands" },
        { title: "TUI Mode", slug: "tui" },
        { title: "Targets & Plugins", slug: "targets" },
      ],
    },
    {
      title: "Safety & Configuration",
      children: [
        { title: "Git Protection", slug: "safety" },
        { title: "Configuration", slug: "configuration" },
      ],
    },
  ],

  defaultPage: "introduction",

  footer: {
    left: "Released under the WTFPL License",
    right: "null-e — Developer disk cleanup CLI",
  },
};
