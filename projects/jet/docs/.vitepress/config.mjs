import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'cclab',
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
