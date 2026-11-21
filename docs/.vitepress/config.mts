import { defineConfig } from 'vitepress'
// Custom theme is auto-loaded from .vitepress/theme/index.mts

export default defineConfig({
  title: 'OXCORD',
  description: 'Next-Generation Modular & High-Performance Rust Discord Library',
  head: [
    ['link', { rel: 'icon', type: 'image/png', href: '/favicon.png' }],
    ['meta', { name: 'theme-color', content: '#5865F2' }]
  ],
  themeConfig: {
    logo: { light: '/icon-light-transparent.png', dark: '/icon-transparent.png', alt: 'OXCORD Logo' },
    siteTitle: 'OXCORD',
    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Architecture', link: '/guide/architecture' },
      { text: 'Migration', link: '/guide/migrating-from-serenity' },
      { text: 'API Reference', link: 'https://docs.rs/oxcord' }
    ],
    sidebar: [
      {
        text: 'Overview',
        items: [
          { text: 'What is OXCORD?', link: '/guide/what-is-oxcord' },
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Modular Architecture', link: '/guide/architecture' }
        ]
      },
      {
        text: 'Core Features',
        items: [
          { text: 'Slash Commands & Modals', link: '/guide/slash-commands' },
          { text: 'Modern Voice (v8 & AEAD)', link: '/guide/voice' },
          { text: 'Pluggable State & Cache', link: '/guide/cache' },
          { text: 'Distributed Message Broker', link: '/guide/broker' },
          { text: 'Zero-Alloc Webhooks', link: '/guide/webhooks' },
          { text: 'Gateway & Zstd Compression', link: '/guide/gateway' }
        ]
      },
      {
        text: 'Migration Guides',
        items: [
          { text: 'From Serenity', link: '/guide/migrating-from-serenity' },
          { text: 'From Twilight', link: '/guide/migrating-from-twilight' }
        ]
      },
      {
        text: 'Examples',
        items: [
          { text: 'Basic Ping Bot (e01)', link: '/guide/examples/e01-basic-ping' },
          { text: 'Slash Commands (e14)', link: '/guide/slash-commands' },
          { text: 'Voice Playback (e21)', link: '/guide/voice' },
          { text: 'Distributed Cache Bot (e22)', link: '/guide/examples/e22-distributed-cache' },
          { text: 'Message Broker Bot (e23)', link: '/guide/broker' },
          { text: 'Stateless Worker (e24)', link: '/guide/broker' },
          { text: 'Full Enterprise Bot (e25)', link: '/guide/broker' }
        ]
      }
    ],
    search: {
      provider: 'local'
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/wumx-labs/oxcord' },
      { icon: 'discord', link: 'https://discord.gg/JnTRems5CT' }
    ],
    footer: {
      message: 'Released under Apache-2.0 License.',
      copyright: 'Copyright © 2026 NoxlyDev & Oxcord Contributors'
    }
  }
})
