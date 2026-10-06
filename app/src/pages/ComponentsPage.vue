<script setup lang="ts">
// Component gallery — every UI building block the app actually ships:
// rendered families + the full stylesheet class inventory.
import { ref } from 'vue'
import {
  Accordion, Alert, AspectRatio, Avatar, AvatarGroup, Badge, BarChart, Breadcrumb,
  BudgetCards, ButtonGroup, CalendarGrid, CardBase, CarouselTabs, Checkbox, Chip, CodeBlock,
  Collapsible, Combobox, ContextMenu, CopyBar, DatePickerPopup, DropdownMenu, EmptyState,
  ErrorBoundary, FeedGrid, Field, GuideHero, HashtagGroups, HoverCard, HubTabs, InfoTip,
  Input, InputGroup, Kbd, KpiCard, Label, ListItem, Menubar, MonthToggle, NavigationMenu,
  Pagination, PinInput, Popover, ProgressBar, RadioGroup, Resizable, ScrollArea, Select,
  Separator, Skeleton, Slider, SocialMediaBar, Spinner, StatCards, StatusChip, StatusDot,
  StatusFunnel, Switch, Tabs, Textarea, Toast, Toggle, ToggleGroup, Tooltip,
} from '@/components/ui'
import Sidebar from '@/components/layout/Sidebar.vue'
import CommandPalette from '@/components/overlays/CommandPalette.vue'
import ConfirmModal from '@/components/overlays/ConfirmModal.vue'
import Dialog from '@/components/overlays/Dialog.vue'
import Drawer from '@/components/overlays/Drawer.vue'
import type { MenuItem } from '@/core/menu'
import MasterTable from '@/components/tables/MasterTable.vue'
import TxnTable from '@/components/tables/TxnTable.vue'
import { FEED_META, type FeedPlatformId } from '@/core/platforms'
import type { HashtagGroup, Post, Status, Txn } from '@/mock/db'

const month = ref(6)
const hubTab = ref('a')
const hubTabs = [ { id: 'a', label: 'Alpha' }, { id: 'b', label: 'Beta' }, { id: 'c', label: 'Gamma' } ]
const carousel = ref('Two')
const carouselItems = ['One', 'Two', 'Three']
const date = ref<string | null>('2026-12-13')
const statuses: Status[] = ['Start', 'Design', 'Dev', 'Done']
const bars = [ { label: 'Mon', value: 3 }, { label: 'Tue', value: 6 }, { label: 'Wed', value: 4 }, { label: 'Thu', value: 8 } ]
const seg = ref('soft')
const demoPosts: Post[] = [
  { id: 'p-1', month: 12, topic: 'Morning routine', pillar: 'Pillar I', format: 'Reel', goal: 'Reach', date: '2026-12-13', time: '09:00', status: 'Design', hook: '', caption: '', cta: '', hashtagGroup: 'Niche', hashtags: [], imageUrl: '', note: '', done: false, platforms: ['Facebook'] },
  { id: 'p-2', month: 12, topic: 'Desk setup', pillar: 'Pillar II', format: 'Carousel', goal: 'Engage', date: '2026-12-14', time: '18:00', status: 'Done', hook: '', caption: '', cta: '', hashtagGroup: 'Reach', hashtags: [], imageUrl: '', note: '', done: true, platforms: ['Instagram'] },
]
const demoTxns: Txn[] = [
  { id: 't-1', date: '2026-01-05', amount: 15000, kind: 'IN', category: 'Sponsorship', sub: 'Brand A' },
  { id: 't-2', date: '2026-01-09', amount: 2500, kind: 'OUT', category: 'Ads', sub: 'Boost' },
]
const demoGroups: HashtagGroup[] = [
  { id: 'g-niche', title: 'Niche', tags: ['selfcare', 'budget', 'desksetup'] },
  { id: 'g-reach', title: 'Reach', tags: ['morning', 'vlog'] },
]
const feedPlatform: FeedPlatformId = 'meta'
const page = ref(1)
const confirmOpen = ref(false)
const statItems = [
  { label: 'Posted', value: 128 },
  { label: 'Reach', value: '24.1K' },
  { label: 'Saved', value: 12 },
]
const demoInput = ref('')
const demoText = ref('')
const demoField = ref('')
const demoIgroup = ref('huuk.app')
const demoCheckA = ref(true)
const demoCheckInd = ref(false)
const demoRadio = ref('b')
const demoSwitch = ref(true)
const demoSlider = ref(40)
const demoPin = ref('1234')
const demoSelect = ref<string | undefined>('pro')
const demoCombo = ref('')
const demoTabs = ref('overview')
const demoAcc = ref<string[]>(['a'])
const demoCollapse = ref(true)
const demoNav = ref('home')
const demoMenu = ref(false)
const demoPop = ref(false)
const demoCmd = ref(false)
const demoDialog = ref(false)
const demoDrawer = ref(false)
const demoToast = ref(true)
const demoSidebar = ref('home')
const labelOptions = [
  { value: 'free', label: 'Free' },
  { value: 'pro', label: 'Pro' },
  { value: 'team', label: 'Team', disabled: true },
]
const comboboxOptions = [
  { value: 'vue', label: 'Vue' },
  { value: 'react', label: 'React' },
  { value: 'svelte', label: 'Svelte' },
]
const tabItems = [
  { id: 'overview', label: 'Overview' },
  { id: 'posts', label: 'Posts', badge: 12 },
  { id: 'locked', label: 'Locked', disabled: true },
]
const accItems = [
  { id: 'a', title: 'What is Huuk?' },
  { id: 'b', title: 'Billing & plans' },
  { id: 'c', title: 'Workspace roles' },
]
const navItems = [
  { id: 'home', label: 'Home', href: '#components' },
  {
    id: 'plan',
    label: 'Plan',
    children: [
      { id: 'calendar', label: 'Calendar', href: '#components', description: 'Schedule by day' },
      { id: 'ideas', label: 'Ideas', href: '#components', description: 'Capture sparks' },
    ],
  },
]
const menuItems: MenuItem[] = [
  { id: 'edit', label: 'Edit' },
  { id: 'duplicate', label: 'Duplicate' },
  { id: 'delete', label: 'Delete', danger: true, separatorBefore: true },
]
const menubarMenus = [
  { id: 'file', label: 'File', items: [{ id: 'new', label: 'New' }, { id: 'open', label: 'Open' }] },
  { id: 'edit', label: 'Edit', items: [{ id: 'undo', label: 'Undo' }, { id: 'redo', label: 'Redo', disabled: true }] },
]
const commandItems = [
  { id: 'new-post', label: 'New post', group: 'Create', shortcut: '⌘N' },
  { id: 'brand', label: 'Open Brand kit', group: 'Navigate' },
  { id: 'invite', label: 'Invite member', group: 'Team', shortcut: '⌘I' },
]
const sidebarItems = [
  { id: 'home', label: 'Home', icon: '⌂' },
  { id: 'plan', label: 'Plan', icon: '▦' },
  { id: 'content', label: 'Content', icon: '✎' },
  { id: 'settings', label: 'Settings', icon: '⚙' },
]

const CLASSES: string[] = [
  'actions',
  'active',
  'ads-input',
  'ads-link',
  'ads-stat',
  'app',
  'autherr',
  'avatar',
  'avatar-menu',
  'avatar-wrap',
  'backdrop',
  'badge-danger',
  'badge-success',
  'badge-warning',
  'bar',
  'barlabels',
  'bars',
  'body',
  'brand',
  'brand-body',
  'brand-head',
  'brand-home',
  'brand-logo',
  'brand-nav',
  'brand-note',
  'brand-prompt',
  'brand-step',
  'brandpage',
  'btn',
  'btn-primary',
  'cal',
  'camp-editor',
  'camp-grid',
  'camp-item',
  'camp-item-head',
  'camp-list',
  'camp-status',
  'card',
  'card-scroll',
  'cards',
  'cell',
  'chip',
  'chipbtn',
  'chiprow',
  'completed',
  'conn-btn',
  'conn-name',
  'conn-row',
  'connected',
  'connecting',
  'cover',
  'danger',
  'dark',
  'day',
  'daynum',
  'deck',
  'disabled',
  'done',
  'dot',
  'dow',
  'dp-caret',
  'dp-day',
  'dp-dow',
  'dp-field',
  'dp-grid',
  'dp-head',
  'dp-modal',
  'dp-nav',
  'dragging',
  'editable',
  'empty',
  'error',
  'expand-btn',
  'fading',
  'feature-tag',
  'feed',
  'field',
  'font-check',
  'font-choice',
  'font-grid',
  'font-import',
  'font-import-plus',
  'font-name',
  'font-pick',
  'font-preview',
  'font-preview-sample',
  'font-remove',
  'font-sample',
  'footer',
  'full',
  'gal-eyebrow',
  'gal-grid',
  'gal-list',
  'gallery',
  'gcap',
  'ghost',
  'gitem',
  'grid2',
  'grid3',
  'grid4',
  'grid6',
  'guide-head',
  'guide-overlay',
  'guide-panel',
  'heroimg',
  'hollow',
  'identity-badge',
  'identity-card',
  'identity-download',
  'identity-hint',
  'identity-meta',
  'identity-preview',
  'img-bounds',
  'infotip',
  'infotip-bubble',
  'infotip-wrap',
  'is-primary',
  'lang',
  'lbl',
  'linkgrid',
  'linkitem',
  'links',
  'livecard',
  'lockchip',
  'loginpage',
  'logo-preview',
  'logo-remove',
  'logo-slot',
  'lp-brand',
  'lp-by',
  'lp-card',
  'lp-hint',
  'lp-lang',
  'lp-logo',
  'lp-submit',
  'lp-switch',
  'lp-ws',
  'main',
  'member-row',
  'menubtn',
  'modal',
  'month-toggle',
  'mood-add',
  'mood-fix',
  'mood-item',
  'moodboard',
  'mt',
  'mt-btn',
  'mt-label',
  'muted',
  'navdate',
  'navdrop',
  'navgroup',
  'navlink',
  'navmenu',
  'navmenu-link',
  'num',
  'oauth-banner',
  'oknote',
  'on',
  'open',
  'oversize-fix',
  'palette-caption',
  'palette-card',
  'palette-copy',
  'palette-field',
  'palette-grid',
  'palette-hex',
  'palette-meta',
  'palette-stats',
  'palette-strip',
  'palette-tile',
  'palette-tile-label',
  'palette-tile-role',
  'palette-tile-sample',
  'palette-tile-star',
  'panel',
  'panel-note',
  'paused',
  'pcard',
  'ph',
  'pick-list',
  'pick-row',
  'plan-badge',
  'plan-card',
  'plan-cta',
  'plan-current',
  'plan-list',
  'plan-price',
  'plan-tag',
  'planner-bar',
  'planner-page',
  'plans-card',
  'plans-foot',
  'plans-grid',
  'plans-head',
  'plans-sub',
  'planspage',
  'platbtn',
  'platform-wrap',
  'platmenu',
  'platrow',
  'plogin',
  'plogin-actions',
  'plogin-body',
  'plogin-brand',
  'plogin-card',
  'plogin-close',
  'plogin-full',
  'plogin-note',
  'plogin-title',
  'pop-actions',
  'pop-block',
  'pop-connect',
  'popular',
  'read-article',
  'router-link-active',
  'row',
  'sc-add',
  'sc-bigdate',
  'sc-daterow',
  'sc-day',
  'sc-day-dd',
  'sc-day-left',
  'sc-day-mm',
  'sc-day-wd',
  'sc-dd',
  'sc-event',
  'sc-hour',
  'sc-min',
  'sc-mon',
  'sc-month',
  'sc-navtext',
  'sc-pill',
  'sc-seg',
  'sc-sep',
  'sc-slot',
  'sc-task',
  'sc-task-foot',
  'sc-task-title',
  'sc-tasks',
  'sc-taskshead',
  'sc-time',
  'sc-timeline',
  'sc-times',
  'sc-top',
  'sc-weekday',
  'selected',
  'settings-body',
  'shellgrid',
  'side',
  'slide',
  'slide-card',
  'smartcal',
  'socialbar',
  'socialbar-manage',
  'socialbar-title',
  'socialcheck',
  'socialmini',
  'socialname',
  'sortable',
  'spacer',
  'status-dot',
  'statuschip',
  'step',
  'step-mark',
  'steps',
  'steps-list',
  'strip-bar',
  'strip-bars',
  'strip-btn',
  'strip-label',
  'strip-link',
  'style-export',
  'style-label',
  'style-num',
  'style-panel',
  'style-range',
  'style-row',
  'style-sample',
  'style-segmented',
  'style-value',
  'surface-raised',
  'sw',
  'switchlink',
  'tab',
  'tabs',
  'tbl',
  'tblwrap',
  'today',
  'toolbtn',
  'topbar',
  'warnbox',
  'wide',
  'wire',
  'wire-edit',
  'ws-actions',
  'ws-caret',
  'ws-check',
  'ws-count',
  'ws-form',
  'ws-head',
  'ws-item',
  'ws-menu',
  'ws-name',
  'ws-trigger',
  'ws-wrap'
]
const INVENTORY: [string, string, string][] = [
  ['Accordion', 'components/ui/Accordion.vue', 'disclosure list (single/multiple)'],
  ['Alert', 'components/ui/Alert.vue', 'inline notice, 4 tones · merged from Pugu'],
  ['AppShell', 'components/layout/AppShell.vue', 'navbar shell + overlays'],
  ['AspectRatio', 'components/ui/AspectRatio.vue', 'fixed-ratio media box'],
  ['Avatar', 'components/ui/Avatar.vue', 'square initials chip · merged from Pugu'],
  ['AvatarGroup', 'components/ui/AvatarGroup.vue', 'overlapping avatars + +N · merged from Pugu'],
  ['Badge', 'components/ui/Badge.vue', 'status badge, 5 variants'],
  ['BarChart', 'components/ui/BarChart.vue', 'bar chart (label/value[])'],
  ['Breadcrumb', 'components/ui/Breadcrumb.vue', 'path trail'],
  ['BudgetCards', 'components/ui/BudgetCards.vue', 'income / expense / balance'],
  ['ButtonGroup', 'components/ui/ButtonGroup.vue', 'attached button row'],
  ['CalendarGrid', 'components/ui/CalendarGrid.vue', 'month grid WEEK 1–6'],
  ['CardBase', 'components/ui/CardBase.vue', 'media card base'],
  ['CarouselTabs', 'components/ui/CarouselTabs.vue', 'snap tab carousel'],
  ['ChangePasswordModal', 'components/overlays/ChangePasswordModal.vue', 'change password modal'],
  ['Checkbox', 'components/ui/Checkbox.vue', 'v-model boolean + indeterminate'],
  ['Chip', 'components/ui/Chip.vue', 'label pill'],
  ['CodeBlock', 'components/ui/CodeBlock.vue', 'code surface + lang label · merged from Pugu'],
  ['Collapsible', 'components/ui/Collapsible.vue', 'show/hide section'],
  ['Combobox', 'components/ui/Combobox.vue', 'filterable select'],
  ['CommandPalette', 'components/overlays/CommandPalette.vue', '⌘K search dialog'],
  ['ConfirmModal', 'components/overlays/ConfirmModal.vue', 'confirm dialog · merged from Pugu'],
  ['ContentEditor', 'components/editor/ContentEditor.vue', 'CMS editor'],
  ['ContextMenu', 'components/ui/ContextMenu.vue', 'right-click menu'],
  ['CopyBar', 'components/ui/CopyBar.vue', 'copy-to-clipboard bar'],
  ['DatePickerPopup', 'components/ui/DatePickerPopup.vue', 'field + month calendar'],
  ['Dialog', 'components/overlays/Dialog.vue', 'generic modal dialog'],
  ['Drawer', 'components/overlays/Drawer.vue', 'side/bottom sheet'],
  ['DropdownMenu', 'components/ui/DropdownMenu.vue', 'click menu with items'],
  ['EmptyState', 'components/ui/EmptyState.vue', 'dashed empty placeholder · merged from Pugu'],
  ['ErrorBoundary', 'components/ui/ErrorBoundary.vue', 'catches descendant errors'],
  ['FeedGrid', 'components/ui/FeedGrid.vue', '3×3 feed preview'],
  ['Field', 'components/ui/Field.vue', 'label + control + hint/error'],
  ['GuideHero', 'components/ui/GuideHero.vue', 'onboarding guide content'],
  ['HashtagGroups', 'components/ui/HashtagGroups.vue', 'hashtag groups + add'],
  ['HoverCard', 'components/ui/HoverCard.vue', 'rich hover panel'],
  ['HubTabs', 'components/ui/HubTabs.vue', 'hub tab bar (pill)'],
  ['InfoTip', 'components/ui/InfoTip.vue', 'info button popover'],
  ['Input', 'components/ui/Input.vue', 'v-model text input, 3 sizes'],
  ['InputGroup', 'components/ui/InputGroup.vue', 'prefix/suffix affixes'],
  ['Kbd', 'components/ui/Kbd.vue', 'keyboard key'],
  ['KpiCard', 'components/ui/KpiCard.vue', 'KPI number card'],
  ['Label', 'components/ui/Label.vue', 'form label + required marker'],
  ['ListItem', 'components/ui/ListItem.vue', 'leading/main/trailing row'],
  ['LiveSection', 'components/live/LiveSection.vue', 'live mirror section'],
  ['LoginModal', 'components/overlays/LoginModal.vue', 'login / register modal'],
  ['MasterTable', 'components/tables/MasterTable.vue', 'TanStack table (search/sort/select)'],
  ['Menubar', 'components/ui/Menubar.vue', 'desktop menu bar'],
  ['MonthToggle', 'components/ui/MonthToggle.vue', '‹ Month › toggle'],
  ['NavigationMenu', 'components/ui/NavigationMenu.vue', 'nav with dropdown panels'],
  ['Pagination', 'components/ui/Pagination.vue', 'page info + prev/next · merged from Pugu'],
  ['PinInput', 'components/ui/PinInput.vue', 'OTP cells + paste'],
  ['PlatformLogin', 'components/overlays/PlatformLogin.vue', 'full-screen platform connect'],
  ['Popover', 'components/ui/Popover.vue', 'anchored floating panel'],
  ['ProfilePanel', 'components/overlays/ProfilePanel.vue', 'profile panel'],
  ['ProgressBar', 'components/ui/ProgressBar.vue', 'thin progress track · merged from Pugu'],
  ['ProtectedModal', 'components/overlays/ProtectedModal.vue', 'computed-cell guard'],
  ['RadioGroup', 'components/ui/RadioGroup.vue', 'v-model radio options'],
  ['Resizable', 'components/ui/Resizable.vue', 'drag-split two panes'],
  ['ScreensDeck', 'components/layout/ScreensDeck.vue', 'screen deck navigation'],
  ['ScrollArea', 'components/ui/ScrollArea.vue', 'styled scroll container'],
  ['Select', 'components/ui/Select.vue', 'custom dropdown select'],
  ['Separator', 'components/ui/Separator.vue', 'divider + labelled'],
  ['SettingsPanel', 'components/overlays/SettingsPanel.vue', 'workspace / permissions'],
  ['Sidebar', 'components/layout/Sidebar.vue', 'collapsible side nav'],
  ['Skeleton', 'components/ui/Skeleton.vue', 'loading placeholder · merged from Pugu'],
  ['Slider', 'components/ui/Slider.vue', 'v-model range'],
  ['SocialMediaBar', 'components/ui/SocialMediaBar.vue', 'platform include/connect'],
  ['Spinner', 'components/ui/Spinner.vue', 'loading spinner'],
  ['StatCards', 'components/ui/StatCards.vue', 'label/value stat tiles · merged from Pugu'],
  ['StatusChip', 'components/ui/StatusChip.vue', 'status chip'],
  ['StatusDot', 'components/ui/StatusDot.vue', 'on/off indicator dot · merged from Pugu'],
  ['StatusFunnel', 'components/ui/StatusFunnel.vue', 'Start → Design → Dev → Done'],
  ['Switch', 'components/ui/Switch.vue', 'v-model toggle'],
  ['Tabs', 'components/ui/Tabs.vue', 'v-model tab bar + badge'],
  ['Textarea', 'components/ui/Textarea.vue', 'v-model multiline'],
  ['Toast', 'components/ui/Toast.vue', 'auto-dismiss notice, 4 tones'],
  ['Toggle', 'components/ui/Toggle.vue', 'pressed button'],
  ['ToggleGroup', 'components/ui/ToggleGroup.vue', 'single-choice toggle row'],
  ['Tooltip', 'components/ui/Tooltip.vue', 'hover/focus text tip'],
  ['TxnTable', 'components/tables/TxnTable.vue', 'transactions table']
]

const PAGES: [string, string, string][] = [
  ['DashboardPage', 'pages/DashboardPage.vue', '/dashboard (Home)'],
  ['PlanPage', 'pages/PlanPage.vue', '/plan (hub)'],
  ['PlannerPage', 'pages/PlannerPage.vue', 'Plan › Monthly'],
  ['CalendarPage', 'pages/CalendarPage.vue', 'Plan › Calendar'],
  ['IdeasPage', 'pages/IdeasPage.vue', 'Plan › Ideas'],
  ['HashtagsPage', 'pages/HashtagsPage.vue', 'Plan › Hashtags'],
  ['ContentPage', 'pages/ContentPage.vue', '/content (hub)'],
  ['StudioPage', 'pages/StudioPage.vue', 'Content › Studio'],
  ['FeedPage', 'pages/FeedPage.vue', 'Content › Feed preview'],
  ['PromotePage', 'pages/PromotePage.vue', '/promote (hub)'],
  ['CampaignsPage', 'pages/CampaignsPage.vue', 'Promote › Campaigns'],
  ['AdsPage', 'pages/AdsPage.vue', 'Promote › Meta Ads'],
  ['AnalyzePage', 'pages/AnalyzePage.vue', '/analyze (hub)'],
  ['PerformancePage', 'pages/PerformancePage.vue', 'Analyze › Performance'],
  ['FinancePage', 'pages/FinancePage.vue', 'Analyze › Finance'],
  ['SettingsPage', 'pages/SettingsPage.vue', '/settings (hub)'],
  ['BrandPage', 'pages/BrandPage.vue', 'Settings › Brand'],
  ['MembersPage', 'pages/MembersPage.vue', 'Settings › Members'],
  ['LoginPage', 'pages/LoginPage.vue', '/login'],
  ['RegisterPage', 'pages/RegisterPage.vue', '/register'],
  ['PlansPage', 'pages/PlansPage.vue', '/plans'],
  ['WorkspaceSetupPage', 'pages/WorkspaceSetupPage.vue', '/welcome'],
  ['ReadPage', 'pages/ReadPage.vue', '/read · /read/:slug'],
  ['NotFoundPage', 'pages/NotFoundPage.vue', '404'],
  ['ComponentsPage', 'pages/ComponentsPage.vue', '/components'],
]

</script>

<template>
  <div class="gallery">
    <header class="gal-head">
      <span class="gal-eyebrow">huuk · design system</span>
      <h1>Components</h1>
      <p class="muted">ทุก UI building block ที่แอปใช้จริง — เรนเดอร์เป็นหมวด + class ทั้งหมดใน stylesheet</p>
    </header>

    <section class="gal-section">
      <h2>Buttons &amp; actions</h2>
      <div class="gal-grid">
        <div class="gitem"><button class="btn btn-primary">Primary</button><span class="gcap">btn btn-primary</span></div>
        <div class="gitem"><button class="btn">Ghost</button><span class="gcap">btn (ghost)</span></div>
        <div class="gitem"><button class="btn danger">Delete</button><span class="gcap">btn danger</span></div>
        <div class="gitem"><button class="btn" disabled>Disabled</button><span class="gcap">btn :disabled</span></div>
        <div class="gitem"><button class="btn btn-primary"><span class="gcap" style="margin:0">＋ </span>New</button><span class="gcap">btn with icon</span></div>
        <div class="gitem"><button class="menubtn">Menu item</button><span class="gcap">menubtn</span></div>
        <div class="gitem"><button class="toolbtn">B</button><span class="gcap">toolbtn</span></div>
        <div class="gitem"><button class="switchlink">switch link</button><span class="gcap">switchlink</span></div>
        <div class="gitem"><button class="chipbtn on">chipbtn</button><span class="gcap">chipbtn</span></div>
        <div class="gitem"><button class="btn socialmini">mini</button><span class="gcap">socialmini</span></div>
        <div class="gitem"><button class="btn expand-btn">Go full page</button><span class="gcap">expand-btn</span></div>
        <div class="gitem"><button class="btn logo-remove">×</button><span class="gcap">logo-remove</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Form controls</h2>
      <div class="gal-grid">
        <div class="gitem"><input class="field" placeholder="Text" /><span class="gcap">field (text)</span></div>
        <div class="gitem"><input class="field" type="email" placeholder="Email" /><span class="gcap">field (email)</span></div>
        <div class="gitem"><input class="field" type="password" value="secret" /><span class="gcap">field (password)</span></div>
        <div class="gitem"><input class="field" type="number" value="12" /><span class="gcap">field (number)</span></div>
        <div class="gitem"><input class="field" type="date" /><span class="gcap">field (date)</span></div>
        <div class="gitem"><input class="field" type="time" value="09:00" /><span class="gcap">field (time)</span></div>
        <div class="gitem"><input class="field" type="search" placeholder="Search…" /><span class="gcap">search</span></div>
        <div class="gitem"><input class="field" type="range" min="0" max="24" /><span class="gcap">style-range</span></div>
        <div class="gitem"><input class="field" type="color" value="#000000" /><span class="gcap">color picker</span></div>
        <div class="gitem"><select class="field"><option>Option A</option><option>Option B</option></select><span class="gcap">select.field</span></div>
        <div class="gitem"><textarea class="field" rows="2" placeholder="Textarea"></textarea><span class="gcap">textarea.field</span></div>
        <div class="gitem"><label class="btn">Upload<input type="file" hidden /></label><span class="gcap">file upload</span></div>
        <div class="gitem"><input class="field palette-hex" placeholder="#4f46e5" /><span class="gcap">palette-hex</span></div>
        <div class="gitem"><input class="field style-num" type="number" value="12" /><span class="gcap">style-num</span></div>
        <div class="gitem"><label class="lbl">Label</label><input class="field" placeholder="with label" /><span class="gcap">lbl + field</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Selection</h2>
      <div class="gal-grid">
        <div class="gitem"><label class="muted"><input type="checkbox" checked /> Checkbox</label><span class="gcap">checkbox</span></div>
        <div class="gitem"><label class="muted"><input type="radio" name="g" checked /> Radio A</label><label class="muted"><input type="radio" name="g" /> Radio B</label><span class="gcap">radio</span></div>
        <div class="gitem">
          <div class="style-segmented" role="group"><button :class="{ on: seg === 'none' }" @click="seg = 'none'">None</button><button :class="{ on: seg === 'soft' }" @click="seg = 'soft'">Soft</button><button :class="{ on: seg === 'strong' }" @click="seg = 'strong'">Strong</button></div>
          <span class="gcap">style-segmented</span>
        </div>
        <div class="gitem"><div class="tabs"><button class="tab">One</button><button class="tab active">Two</button><button class="tab">Three</button></div><span class="gcap">tabs</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Navigation</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2">
          <div class="topbar" style="position: static; border: 1px solid var(--line); gap: 14px; padding: 10px 14px; backdrop-filter: none; width: 100%">
            <span class="brand"><span class="brand-home">Huuk</span></span>
            <span class="links"><a class="navlink active">Home</a><a class="navlink">Plan</a><a class="navlink">Content</a></span>
            <span class="side"><span class="lang">EN</span><span class="avatar">P</span></span>
          </div>
          <span class="gcap">topbar · brand · navlink · avatar · lang</span>
        </div>
        <div class="gitem">
          <div class="navmenu" style="position: static; box-shadow: none">
            <a class="navmenu-link active">Monthly</a><a class="navmenu-link">Calendar</a><a class="navmenu-link">Ideas</a>
          </div>
          <span class="gcap">navmenu / navmenu-link</span>
        </div>
        <div class="gitem">
          <span class="wire">workspace</span>
          <span class="ws-caret">▾</span>
          <span class="gcap">wire (workspace trigger)</span>
        </div>
        <div class="gitem">
          <div class="ws-menu" style="position: static; box-shadow: none">
            <div class="ws-head">Workspaces</div>
            <button class="ws-item active"><span class="ws-name">Teenueng</span><span class="ws-count">3/4</span><span class="ws-check">✓</span></button>
          </div>
          <span class="gcap">ws-menu / ws-item / ws-check</span>
        </div>
        <div class="gitem">
          <div class="avatar-menu" style="position: static; box-shadow: none">
            <div class="pop-connect"><div class="ws-head">Social Media</div><div class="conn-row"><span class="status-dot connected"></span><span class="conn-name">Meta</span><button class="conn-btn">Manage</button></div></div>
            <div class="pop-actions"><button class="menubtn">Workspace settings</button><button class="menubtn">Log out</button></div>
          </div>
          <span class="gcap">avatar-menu / conn-row / pop-actions</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Chips, tags &amp; badges</h2>
      <div class="gal-grid">
        <div class="gitem"><Chip label="chip" /><Chip label="active" dark /><span class="gcap">chip / chip.dark</span></div>
        <div class="gitem"><span class="feature-tag">feature</span><span class="gcap">feature-tag</span></div>
        <div class="gitem"><span class="badge badge-success">Success</span><span class="badge badge-warning">Warning</span><span class="badge badge-danger">Danger</span><span class="gcap">badge-*</span></div>
        <div class="gitem"><span class="statuschip st-published">Published</span><span class="statuschip st-draft">Draft</span><span class="gcap">statuschip</span></div>
        <div class="gitem"><span class="badge badge-success">Badge</span><span class="plan-badge">Most popular</span><span class="gcap">plan-badge</span></div>
        <div class="gitem"><span class="chiprow"><button class="chipbtn on">Pillar I</button><button class="chipbtn">Pillar II</button></span><span class="gcap">chiprow</span></div>
        <div class="gitem"><span class="lockchip">Locked by Pooh</span><span class="gcap">lockchip</span></div>
        <div class="gitem"><Avatar label="P" /><Avatar label="K" /><Avatar label="+4" more /><span class="gcap">Avatar</span></div>
        <div class="gitem"><AvatarGroup :items="['P', 'K', 'M', 'S', 'A']" :max="3" /><span class="gcap">AvatarGroup</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Status</h2>
      <div class="gal-grid">
        <div class="gitem"><span class="status-dot connected"></span> connected <span class="status-dot connecting"></span> connecting <span class="status-dot"></span> off<span class="gcap">status-dot states</span></div>
        <div class="gitem">
          <span v-for="s in statuses" :key="s"><StatusChip :status="s" /></span>
          <span class="gcap">StatusChip</span>
        </div>
        <div class="gitem"><StatusFunnel :current="'Design'" /><span class="gcap">StatusFunnel</span></div>
        <div class="gitem"><span class="camp-status active">active</span><span class="camp-status draft">draft</span><span class="camp-status paused">paused</span><span class="gcap">camp-status</span></div>
        <div class="gitem"><span style="display:inline-flex;align-items:center;gap:8px"><StatusDot on title="on" /> on</span><span style="display:inline-flex;align-items:center;gap:8px"><StatusDot title="off" /> off</span><span class="gcap">StatusDot</span></div>
        <div class="gitem" style="width:100%"><ProgressBar :value="64" /><span class="gcap">ProgressBar :value="64"</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Feedback &amp; notices</h2>
      <div class="gal-grid">
        <div class="gitem"><p class="autherr" style="margin:0">Something went wrong.</p><span class="gcap">autherr (error)</span></div>
        <div class="gitem"><p class="oknote" style="margin:0">Saved successfully.</p><span class="gcap">oknote (success)</span></div>
        <div class="gitem"><p class="warnbox" style="margin:0">Heads up — check this.</p><span class="gcap">warnbox</span></div>
        <div class="gitem"><p class="muted" style="margin:0">Loading…</p><span class="gcap">muted (loading text)</span></div>
        <div class="gitem"><InfoTip text="Helper text" /><span class="gcap">infotip</span></div>
        <div class="gitem"><span class="oauth-banner" style="position:static; margin:0">Connected to Meta <button class="btn" style="width:auto;padding:2px 10px">close</button></span><span class="gcap">oauth-banner</span></div>
        <div class="gitem"><span class="empty muted">No items yet.</span><span class="gcap">empty</span></div>
        <div class="gitem"><Alert title="Info" tone="info">Neutral inline message.</Alert><span class="gcap">Alert · info</span></div>
        <div class="gitem"><Alert title="Saved" tone="success">Published to Content.</Alert><span class="gcap">Alert · success</span></div>
        <div class="gitem"><Alert title="Warning" tone="warning">Reconnect Meta to keep syncing.</Alert><span class="gcap">Alert · warning</span></div>
        <div class="gitem"><Alert title="Error" tone="danger">Upload failed — retry.</Alert><span class="gcap">Alert · danger</span></div>
        <div class="gitem" style="grid-column: span 2"><CodeBlock lang="bash" code="bun run dev --filter app" /><span class="gcap">CodeBlock</span></div>
        <div class="gitem" style="grid-column: span 2"><EmptyState title="No posts yet" message="Create your first post to see it here."><button class="btn btn-primary" style="width:auto">New post</button></EmptyState><span class="gcap">EmptyState</span></div>
        <div class="gitem"><Skeleton /><Skeleton variant="block" /><span class="gcap">Skeleton · line / block</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Cards &amp; tiles</h2>
      <div class="gal-grid">
        <div class="gitem"><div class="card" style="width:100%"><strong>Card</strong><p class="muted" style="margin:4px 0 0">card surface</p></div><span class="gcap">card</span></div>
        <div class="gitem"><div class="panel" style="width:100%"><strong>Panel</strong></div><span class="gcap">panel</span></div>
        <div class="gitem" style="align-items:stretch"><CardBase title="Card base" meta="meta">body</CardBase><span class="gcap">CardBase / pcard</span></div>
        <div class="gitem" style="align-items:stretch">
          <div class="plan-card"><strong>Pro</strong><p class="plan-tag">For teams</p><p class="plan-price">฿0</p><div class="plan-cta"><button class="btn btn-primary">Choose</button></div></div>
          <span class="gcap">plan-card</span>
        </div>
        <div class="gitem"><div class="ads-stat" style="width:100%"><span class="muted">Spend</span><strong>฿1,200</strong></div><span class="gcap">ads-stat</span></div>
        <div class="gitem"><div class="camp-item" style="width:100%"><span class="camp-item-head"><strong>Launch</strong><span class="camp-status active">active</span></span><span class="muted">2026-12-01 → 2026-12-31</span></div><span class="gcap">camp-item</span></div>
        <div class="gitem" style="grid-column: span 2; align-items: stretch"><StatCards :items="statItems" /><span class="gcap">StatCards</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Brand kit</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2">
          <div class="palette-strip"><span class="strip-label muted">Live preview</span><span class="strip-btn">Button</span><span class="strip-link">Link</span><span class="strip-bars"><span class="strip-bar" style="height:13px"></span><span class="strip-bar" style="height:16px"></span><span class="strip-bar" style="height:19px"></span></span></div>
          <span class="gcap">palette-strip / strip-btn / strip-bars</span>
        </div>
        <div class="gitem" style="align-items:stretch">
          <div class="palette-card is-primary"><label class="palette-tile-label"><span class="palette-tile" style="background:#111"><span class="palette-tile-role">Primary</span><span class="palette-tile-star">★</span><span class="palette-tile-sample">Aa</span></span></label><div class="palette-meta"><p class="palette-caption">buttons, links</p><div class="palette-stats"><input class="field palette-hex" placeholder="#111111" /><button class="palette-copy">Copy</button></div></div></div>
          <span class="gcap">palette-card / palette-tile / palette-copy</span>
        </div>
        <div class="gitem">
          <div class="font-preview"><span class="font-preview-sample">Aa Bb 123 · สวัสดี</span><span class="muted">selected font</span></div>
          <div class="font-grid"><button class="font-choice font-pick active"><span class="font-sample">Aa</span><span class="font-name">Inter</span><span class="font-check">✓</span></button><label class="font-choice font-import"><span class="font-import-plus">+</span><span class="font-name">Import</span></label></div>
          <span class="gcap">font-preview / font-choice / font-import</span>
        </div>
        <div class="gitem">
          <div class="logo-slot"><div class="logo-preview"><span class="muted">Main logo</span></div><div class="row"><label class="btn">Upload image<input type="file" hidden /></label><input class="field" placeholder="https URL" /></div></div>
          <span class="gcap">logo-slot / logo-preview</span>
        </div>
        <div class="gitem" style="align-items:stretch">
          <div class="moodboard"><div class="mood-item"><span style="display:block;width:64px;height:64px;background:var(--wash)"></span><button class="logo-remove">×</button></div><label class="mood-add"><span>Add</span></label></div>
          <span class="gcap">moodboard / mood-item / mood-add</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Data &amp; tables</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: 1 / -1; align-items: stretch"><MasterTable :rows="demoPosts" /><span class="gcap">MasterTable</span></div>
        <div class="gitem" style="grid-column: 1 / -1; align-items: stretch"><TxnTable :rows="demoTxns" currency="฿" /><span class="gcap">TxnTable</span></div>
        <div class="gitem" style="grid-column: 1 / -1; align-items: stretch">
          <div class="tblwrap"><table class="tbl"><thead><tr><th>Platform</th><th>Start</th><th class="num">Goal</th></tr></thead><tbody><tr><td>Facebook</td><td>1,200</td><td class="num">5,000</td></tr><tr class="selected"><td>Instagram</td><td>3,400</td><td class="num">10,000</td></tr></tbody></table></div>
          <span class="gcap">tbl / thead / tr.selected / td.num</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Charts</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2; align-items: stretch"><BarChart :items="bars" /><span class="gcap">BarChart</span></div>
        <div class="gitem"><KpiCard label="Posted" value="128" /><span class="gcap">KpiCard</span></div>
        <div class="gitem"><BudgetCards :income="12000" :expense="4500" :balance="7500" currency="฿" /><span class="gcap">BudgetCards</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Media &amp; feed</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2; align-items: stretch"><FeedGrid :posts="demoPosts" :platform="feedPlatform" /><span class="gcap">FeedGrid · {{ FEED_META[feedPlatform].name }}</span></div>
        <div class="gitem" style="grid-column: span 2; align-items: stretch"><CalendarGrid :year="2026" :month="12" :posts="demoPosts" :week-start="0" :show-pillar="true" :show-platform="true" :show-status="true" /><span class="gcap">CalendarGrid</span></div>
        <div class="gitem"><SocialMediaBar /><span class="gcap">SocialMediaBar</span></div>
        <div class="gitem"><GuideHero /><span class="gcap">GuideHero</span></div>
        <div class="gitem"><HashtagGroups :groups="demoGroups" /><span class="gcap">HashtagGroups</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Controls (Vue)</h2>
      <div class="gal-grid">
        <div class="gitem"><MonthToggle v-model="month" /><span class="gcap">MonthToggle</span></div>
        <div class="gitem"><HubTabs :tabs="hubTabs" v-model="hubTab" /><span class="gcap">HubTabs</span></div>
        <div class="gitem"><CarouselTabs :items="carouselItems" v-model="carousel" /><span class="gcap">CarouselTabs</span></div>
        <div class="gitem"><DatePickerPopup v-model="date" title="Select date" /><span class="gcap">DatePickerPopup</span></div>
        <div class="gitem"><CopyBar text="Huuk — copy me" /><span class="gcap">CopyBar</span></div>
        <div class="gitem"><ErrorBoundary><span class="muted">wrapped content</span></ErrorBoundary><span class="gcap">ErrorBoundary</span></div>
        <div class="gitem" style="grid-column: span 2">
          <Pagination :current-page="page" :total-pages="7" @update:current-page="page = $event" />
          <span class="gcap">Pagination (page {{ page }} / 7)</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Overlays (contained preview)</h2>
      <div class="gal-grid">
        <div class="gitem" style="align-items:stretch"><div class="modal" style="position:static;margin:0"><h3>Protected cell</h3><p class="muted">This field is computed.</p><div class="actions"><button class="btn">Cancel</button><button class="btn btn-primary">Confirm</button></div></div><span class="gcap">modal</span></div>
        <div class="gitem" style="align-items:stretch"><div class="guide-panel" style="position:static;box-shadow:none"><div class="guide-head"><strong>Guide</strong><button class="btn" style="width:auto">Close</button></div><p class="muted">…</p></div><span class="gcap">guide-overlay / guide-panel / guide-head</span></div>
        <div class="gitem" style="align-items:stretch"><div class="dp-modal" style="position:static;box-shadow:none"><div class="dp-head">December 2026</div><div class="dp-grid"><span class="dp-dow">S</span><span class="dp-dow">M</span><span class="dp-day">1</span><span class="dp-day today">2</span></div></div><span class="gcap">dp-modal / dp-head / dp-grid / dp-day</span></div>
        <div class="gitem" style="align-items:stretch"><div class="plogin-card" style="width:100%"><h3 class="plogin-title">Connect Meta</h3><p class="muted">Read-only until you connect.</p><div class="plogin-actions"><button class="btn">Reconnect</button><button class="btn">Sync now</button></div></div><span class="gcap">plogin-card / plogin-title / plogin-actions</span></div>
        <div class="gitem" style="align-items:stretch"><button class="btn btn-primary" @click="confirmOpen = true">Open ConfirmModal</button><span class="gcap">ConfirmModal (live)</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Ads &amp; campaigns</h2>
      <div class="gal-grid">
        <div class="gitem"><label class="ads-field"><span>Daily budget</span><input class="ads-input" type="number" value="20" /></label><span class="gcap">ads-field / ads-input</span></div>
        <div class="gitem" style="align-items:stretch"><ul class="ads-audit muted" style="margin:0"><li>12:00 · owner · pause · cmp-1</li><li>11:30 · owner · budget · cmp-2</li></ul><span class="gcap">ads-audit</span></div>
        <div class="gitem"><span class="ads-line muted">Targeting: TH · BKK</span><span class="gcap">ads-line</span></div>
        <div class="gitem"><button class="ads-toggle">Pause</button><button class="ads-link" style="width:auto">Campaign</button><span class="gcap">ads-toggle / ads-link</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Plans</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2; align-items: stretch">
          <div class="plans-grid"><div class="plan-card"><strong>Free</strong><p class="plan-tag">Getting started</p><p class="plan-price">฿0</p><ul class="plan-list"><li>Studio</li></ul><div class="plan-cta"><button class="btn btn-primary">Continue</button></div></div><div class="plan-card popular"><span class="plan-badge">Most popular</span><strong>Pro</strong><p class="plan-tag">Teams</p><p class="plan-price">Contact</p><div class="plan-cta"><button class="btn">Contact sales</button></div></div></div>
          <span class="gcap">plans-grid / plan-card / plan-badge / plan-list / plan-cta</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Forms &amp; inputs (shadcn-parity)</h2>
      <div class="gal-grid">
        <div class="gitem"><Label>Workspace name</Label><Input v-model="demoInput" placeholder="Huuk Studio" /><span class="gcap">Label + Input</span></div>
        <div class="gitem"><Field label="Email" hint="We never share it." required><Input v-model="demoField" placeholder="you@huuk.app" /></Field><span class="gcap">Field (hint)</span></div>
        <div class="gitem"><Field label="Handle" error="Already taken"><Input v-model="demoField" invalid /></Field><span class="gcap">Field (error + invalid)</span></div>
        <div class="gitem"><Textarea v-model="demoText" placeholder="Caption…" /><span class="gcap">Textarea</span></div>
        <div class="gitem"><InputGroup prefix="https://"><Input v-model="demoIgroup" /></InputGroup><span class="gcap">InputGroup (prefix)</span></div>
        <div class="gitem"><InputGroup suffix=".app"><Input v-model="demoIgroup" /></InputGroup><span class="gcap">InputGroup (suffix)</span></div>
        <div class="gitem"><Checkbox v-model="demoCheckA" label="Publish now" /><Checkbox v-model="demoCheckInd" :indeterminate="true" label="Partial" /><span class="gcap">Checkbox (checked / indeterminate)</span></div>
        <div class="gitem"><RadioGroup v-model="demoRadio" :options="labelOptions" direction="row" /><span class="gcap">RadioGroup</span></div>
        <div class="gitem"><Switch v-model="demoSwitch" label="Auto-sync" /><span class="gcap">Switch</span></div>
        <div class="gitem"><Slider v-model="demoSlider" :show-value="true" /><span class="gcap">Slider</span></div>
        <div class="gitem"><PinInput v-model="demoPin" :length="4" /><span class="gcap">PinInput (OTP)</span></div>
        <div class="gitem"><Select v-model="demoSelect" :options="labelOptions" /><span class="gcap">Select</span></div>
        <div class="gitem"><Combobox v-model="demoCombo" :options="comboboxOptions" placeholder="Search framework…" /><span class="gcap">Combobox</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Display &amp; layout primitives</h2>
      <div class="gal-grid">
        <div class="gitem"><Badge>Neutral</Badge><Badge variant="success" dot>Live</Badge><Badge variant="warning">Pending</Badge><Badge variant="danger">Failed</Badge><Badge variant="solid">Pro</Badge><span class="gcap">Badge · 5 variants</span></div>
        <div class="gitem"><Kbd>⌘</Kbd><Kbd size="sm">K</Kbd><span class="gcap">Kbd</span></div>
        <div class="gitem"><Spinner :size="18" /><span class="gcap">Spinner</span></div>
        <div class="gitem" style="grid-column: span 2"><Breadcrumb :items="[{ label: 'Settings' }, { label: 'Brand' }, { label: 'Palette' }]" /><span class="gcap">Breadcrumb</span></div>
        <div class="gitem"><Separator /><span class="gcap">Separator</span></div>
        <div class="gitem"><Separator label="or" /><span class="gcap">Separator (label)</span></div>
        <div class="gitem" style="grid-column: span 2"><AspectRatio :ratio="16 / 9"><div style="display:flex;align-items:center;justify-content:center;height:100%">16 : 9</div></AspectRatio><span class="gcap">AspectRatio</span></div>
        <div class="gitem" style="grid-column: span 2">
          <ListItem title="Credits" description="120 remaining" active><template #trailing><Badge variant="solid">Pro</Badge></template></ListItem>
          <span class="gcap">ListItem</span>
        </div>
        <div class="gitem"><ButtonGroup><button class="btn">Day</button><button class="btn">Week</button><button class="btn">Month</button></ButtonGroup><span class="gcap">ButtonGroup</span></div>
        <div class="gitem" style="grid-column: span 2">
          <ScrollArea max-height="110px"><div style="height:300px">Scrollable content…</div></ScrollArea>
          <span class="gcap">ScrollArea</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Navigation &amp; disclosure</h2>
      <div class="gal-grid">
        <div class="gitem" style="grid-column: span 2"><Tabs v-model="demoTabs" :tabs="tabItems" /><span class="gcap">Tabs (badge / disabled)</span></div>
        <div class="gitem" style="grid-column: span 2"><Accordion v-model="demoAcc" :items="accItems" multiple><template #default="{ item }">Answer for <strong>{{ item.title }}</strong>.</template></Accordion><span class="gcap">Accordion (multiple)</span></div>
        <div class="gitem"><Collapsible v-model="demoCollapse" title="Advanced options"><p class="muted" style="margin:0">Hidden content revealed.</p></Collapsible><span class="gcap">Collapsible</span></div>
        <div class="gitem"><NavigationMenu :items="navItems" :active="demoNav" @select="demoNav = $event" /><span class="gcap">NavigationMenu (hover panel)</span></div>
        <div class="gitem"><Menubar :menus="menubarMenus" /><span class="gcap">Menubar</span></div>
        <div class="gitem"><ToggleGroup v-model="demoTabs" :options="[{ value: 'overview', label: 'Left' }, { value: 'posts', label: 'Center' }, { value: 'locked', label: 'Right' }]" /><span class="gcap">ToggleGroup</span></div>
        <div class="gitem"><Toggle v-model="demoCollapse">Bold</Toggle><span class="gcap">Toggle</span></div>
        <div class="gitem" style="grid-column: span 2"><Sidebar v-model="demoSidebar" :items="sidebarItems" title="Workspace" /><span class="gcap">Sidebar (collapsible)</span></div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Menus &amp; popups</h2>
      <div class="gal-grid">
        <div class="gitem"><Popover v-model="demoPop"><template #trigger><button class="btn">Open popover</button></template><strong>Filters</strong><p class="muted" style="margin:4px 0 0">Anchored panel content.</p></Popover><span class="gcap">Popover</span></div>
        <div class="gitem"><Tooltip text="Duplicate"><button class="btn">Hover me</button></Tooltip><span class="gcap">Tooltip</span></div>
        <div class="gitem"><HoverCard><template #trigger><span class="wire">@pooh</span></template><strong>Pooh</strong><p class="muted" style="margin:4px 0 0">Creator · 12 workspaces</p></HoverCard><span class="gcap">HoverCard</span></div>
        <div class="gitem"><DropdownMenu v-model="demoMenu" :items="menuItems"><template #trigger><button class="btn">Actions ▾</button></template></DropdownMenu><span class="gcap">DropdownMenu</span></div>
        <div class="gitem"><ContextMenu :items="menuItems"><div style="border:1px dashed var(--line);padding:18px;text-align:center;font-size:13px;color:var(--muted)">Right-click here</div></ContextMenu><span class="gcap">ContextMenu</span></div>
        <div class="gitem"><button class="btn" @click="demoCmd = true">⌘K Command</button><span class="gcap">CommandPalette</span></div>
        <div class="gitem"><button class="btn" @click="demoDialog = true">Open dialog</button><span class="gcap">Dialog</span></div>
        <div class="gitem"><button class="btn" @click="demoDrawer = true">Open drawer</button><span class="gcap">Drawer</span></div>
        <div class="gitem" style="grid-column: span 2"><Toast v-model="demoToast" title="Saved" message="Brand kit updated." tone="success" :duration="0" /><span class="gcap">Toast (auto-dismiss when duration &gt; 0)</span></div>
        <div class="gitem" style="grid-column: span 2">
          <Resizable><template #a><strong>Left pane</strong><p class="muted" style="margin:4px 0 0">Drag the divider.</p></template><template #b><strong>Right pane</strong></template></Resizable>
          <span class="gcap">Resizable</span>
        </div>
      </div>
    </section>

    <section class="gal-section">
      <h2>Component index ({{ INVENTORY.length }})</h2>
      <ul class="gal-list">
        <li v-for="c in INVENTORY" :key="c[0]"><strong>{{ c[0] }}</strong> <code>{{ c[1] }}</code> <span class="muted">— {{ c[2] }}</span></li>
      </ul>
    </section>

    <section class="gal-section">
      <h2>Pages ({{ PAGES.length }})</h2>
      <ul class="gal-list">
        <li v-for="p in PAGES" :key="p[0]"><strong>{{ p[0] }}</strong> <code>{{ p[1] }}</code> <span class="muted">— {{ p[2] }}</span></li>
      </ul>
    </section>

    <section class="gal-section">
      <h2>Full stylesheet inventory ({{ CLASSES.length }} classes)</h2>
      <ul class="gal-classes"><li v-for="c in CLASSES" :key="c">{{ c }}</li></ul>
    </section>

    <p class="muted" style="margin-top: 24px">
      Rendered families above + every class in <code>app/src/app/style.css</code> listed below.
      Components marked “merged from Pugu” were ported into Huuk so both projects share one set.
    </p>

    <ConfirmModal
      :open="confirmOpen"
      title="Delete post?"
      message="This action cannot be undone."
      confirm-text="Delete"
      @cancel="confirmOpen = false"
      @confirm="confirmOpen = false"
    />
    <CommandPalette v-model="demoCmd" :items="commandItems" @select="demoCmd = false" />
    <Dialog v-model="demoDialog" title="Invite teammate" description="They will get an email invite.">
      <Input v-model="demoField" placeholder="teammate@huuk.app" />
      <template #footer>
        <button class="btn" @click="demoDialog = false">Cancel</button>
        <button class="btn btn-primary" @click="demoDialog = false">Send invite</button>
      </template>
    </Dialog>
    <Drawer v-model="demoDrawer" title="Post details" side="right">
      <p class="muted" style="margin:0">Drawer body content.</p>
      <template #footer><button class="btn" @click="demoDrawer = false">Close</button></template>
    </Drawer>
  </div>
</template>
