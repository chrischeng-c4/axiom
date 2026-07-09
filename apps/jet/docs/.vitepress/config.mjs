import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Jet',
  description: 'High-performance development tools built in Rust',
  lang: 'en-US',
  ignoreDeadLinks: false,

  themeConfig: {
    nav: [
      { text: 'Jet', link: '/getting-started' },
    ],

    sidebar: {
      '/': [
        {
          text: 'Jet',
          items: [
            { text: 'Getting Started', link: '/getting-started' },
            { text: 'Package Manager', link: '/package-manager' },
            { text: 'Bundler', link: '/bundler' },
            { text: 'Dev Server', link: '/dev-server' },
            { text: 'Task Runner', link: '/task-runner' },
            { text: 'Configuration', link: '/configuration' },
            { text: 'Workspaces', link: '/workspaces' },
            { text: 'OpenAPI Codegen', link: '/openapi-codegen' },
            { text: 'Library Publishing', link: '/library-publishing' },
            { text: 'Migration from Playwright', link: '/migration-from-playwright' },
          ]
        },
        {
          text: 'Architecture',
          items: [
            { text: 'Project Layout', link: '/architecture/layout' },
            { text: 'Source-Tree Reorg Plan', link: '/architecture/reorg-plan' },
          ]
        },
        {
          text: 'Design Notes',
          items: [
            { text: 'Build Fails Loudly on Unresolved Bare Specifiers', link: '/build-fails-loudly-on-unresolved-bare-specifiers' },
            { text: 'Check Exits Non-Zero While Unimplemented', link: '/check-exits-non-zero-while-unimplemented' },
            { text: 'Dev Server Source Analysis UTF-8 Safety', link: '/dev-server-source-analysis-utf8-safety' },
            { text: 'Layout Box Model -- Slice 7a', link: '/layout-box-slice-7a' },
            { text: 'Wasm Config Accepts Shared Jet Sections', link: '/wasm-config-accept-shared-jet-sections' },
            { text: 'Wasm Transpiler Boolean useState Literals', link: '/wasm-transpiler-boolean-usestate-literals' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/chrischeng-c4/cclab' }
    ],

    search: {
      provider: 'local'
    },

    footer: {
      message: 'Built with Rust.',
    }
  }
})
