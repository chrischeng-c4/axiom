import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'cclab',
  description: 'High-performance development tools built in Rust',
  lang: 'en-US',
  ignoreDeadLinks: false,

  themeConfig: {
    nav: [
      { text: 'Jet', link: '/jet/getting-started' },
    ],

    sidebar: {
      '/jet/': [
        {
          text: 'Jet',
          items: [
            { text: 'Getting Started', link: '/jet/getting-started' },
            { text: 'Package Manager', link: '/jet/package-manager' },
            { text: 'Bundler', link: '/jet/bundler' },
            { text: 'Dev Server', link: '/jet/dev-server' },
            { text: 'Task Runner', link: '/jet/task-runner' },
            { text: 'Configuration', link: '/jet/configuration' },
            { text: 'Workspaces', link: '/jet/workspaces' },
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
