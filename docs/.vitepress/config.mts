import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'th',
  title: 'Huuk Docs',
  description: 'คู่มือใช้งาน Feature Specs แบบ Kiro (ภาษาไทย) + บริบทโปรเจกต์ Huuk',
  base: '/',
  srcExclude: ['**/README.md'],

  themeConfig: {
    siteTitle: 'Huuk Docs',
    logo: '/huuk-logo.svg',

    nav: [
      { text: 'เริ่มที่นี่', link: '/01-feature-specs' },
      { text: 'Walkthrough', link: '/11-hands-on-walkthrough' },
      { text: 'Huuk Overview', link: '/00-huuk-overview' },
    ],

    sidebar: [
      {
        text: 'เริ่มต้น (Start here)',
        collapsed: false,
        items: [
          { text: '01 · คู่มือใช้งาน Feature Specs', link: '/01-feature-specs' },
          { text: '11 · Hands-on Walkthrough', link: '/11-hands-on-walkthrough' },
          { text: '00 · ภาพรวม Huuk', link: '/00-huuk-overview' },
        ],
      },
      {
        text: 'Workflows (วิธีใช้ทีละจอ)',
        collapsed: false,
        items: [
          { text: '02 · Requirements-First', link: '/02-requirements-first' },
          { text: '03 · Design-First', link: '/03-design-first' },
          { text: '07 · Quick Spec / Plan / Analyze', link: '/07-quick-plan-analyze' },
          { text: '08 · Bugfix Specs + PBT', link: '/08-bugfix-specs' },
        ],
      },
      {
        text: 'วิธีเขียน (Authoring guides)',
        collapsed: false,
        items: [
          { text: '04 · EARS Notation', link: '/04-ears' },
          { text: '05 · เขียน design.md', link: '/05-design-doc' },
          { text: '06 · เขียน tasks.md + รัน tasks', link: '/06-tasks-doc' },
        ],
      },
      {
        text: 'อ้างอิง (Reference)',
        collapsed: false,
        items: [
          { text: '09 · Best Practices ฉบับ Huuk', link: '/09-best-practices-huuk' },
          { text: '10 · Glossary + Checklists', link: '/10-glossary-checklists' },
        ],
      },
      {
        text: 'Templates + ตัวอย่าง',
        collapsed: true,
        items: [
          { text: 'requirements template', link: '/templates/requirements-template' },
          { text: 'design template', link: '/templates/design-template' },
          { text: 'tasks template', link: '/templates/tasks-template' },
          { text: 'bugfix template', link: '/templates/bugfix-template' },
          { text: 'ตัวอย่าง: Landing Page (req)', link: '/examples/landing-page/requirements' },
          { text: 'ตัวอย่าง: Landing Page (design)', link: '/examples/landing-page/design' },
          { text: 'ตัวอย่าง: Landing Page (tasks)', link: '/examples/landing-page/tasks' },
        ],
      },
    ],

    search: { provider: 'local' },

    outline: { level: [2, 3], label: 'ในหน้านี้' },
    docFooter: { prev: 'ก่อนหน้า', next: 'ถัดไป' },
    lastUpdated: { text: 'อัปเดตล่าสุด' },
    langMenuLabel: 'ภาษา',
    returnToTopLabel: 'กลับขึ้นบน',
    sidebarMenuLabel: 'สารบัญ',
    darkModeSwitchLabel: 'ธีม',
    lightModeSwitchTitle: 'สลับเป็นโหมดสว่าง',
    darkModeSwitchTitle: 'สลับเป็นโหมดมืด',

    socialLinks: [],
    footer: {
      message: 'Huuk Docs — คู่มือ Feature Specs แบบ Kiro (ภาษาไทย) + บริบทโปรเจกต์ Huuk',
      copyright: 'อ้างอิงวิธีใช้งานจาก kiro.dev/docs/specs/',
    },
  },
})
