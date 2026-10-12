// Export docs/*.md (+ templates + examples) into seed/docs.json for Strapi.
// Run: node scripts/export-docs.mjs   (repo root = ../.. → pass nothing; cwd must be cms/)
// Re-run after editing markdown, then restart Strapi (or edit in admin UI instead).
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const here = path.dirname(fileURLToPath(import.meta.url));
const cmsDir = path.resolve(here, '..');
const docsDir = path.resolve(cmsDir, '..', 'docs');

const SECTION_LABEL = {
  start: 'เริ่มต้น',
  workflows: 'Workflows',
  authoring: 'วิธีเขียน',
  reference: 'อ้างอิง',
  templates: 'Templates + ตัวอย่าง',
  examples: 'Templates + ตัวอย่าง',
};

// [relative md path, slug, section, order]
const PAGES = [
  ['README.md', 'index', 'start', 0],
  ['00-huuk-overview.md', '00-huuk-overview', 'start', 1],
  ['01-feature-specs.md', '01-feature-specs', 'start', 2],
  ['11-hands-on-walkthrough.md', '11-hands-on-walkthrough', 'start', 3],
  ['02-requirements-first.md', '02-requirements-first', 'workflows', 4],
  ['03-design-first.md', '03-design-first', 'workflows', 5],
  ['07-quick-plan-analyze.md', '07-quick-plan-analyze', 'workflows', 6],
  ['08-bugfix-specs.md', '08-bugfix-specs', 'workflows', 7],
  ['04-ears.md', '04-ears', 'authoring', 8],
  ['05-design-doc.md', '05-design-doc', 'authoring', 9],
  ['06-tasks-doc.md', '06-tasks-doc', 'authoring', 10],
  ['09-best-practices-huuk.md', '09-best-practices-huuk', 'reference', 11],
  ['10-glossary-checklists.md', '10-glossary-checklists', 'reference', 12],
  ['templates/requirements-template.md', 'template-requirements', 'templates', 20],
  ['templates/design-template.md', 'template-design', 'templates', 21],
  ['templates/tasks-template.md', 'template-tasks', 'templates', 22],
  ['templates/bugfix-template.md', 'template-bugfix', 'templates', 23],
  ['examples/landing-page/requirements.md', 'example-landing-requirements', 'examples', 30],
  ['examples/landing-page/design.md', 'example-landing-design', 'examples', 31],
  ['examples/landing-page/tasks.md', 'example-landing-tasks', 'examples', 32],
];

function stripFrontmatter(text) {
  if (!text.startsWith('---')) return text;
  const end = text.indexOf('\n---', 3);
  if (end === -1) return text;
  return text.slice(end + 4).replace(/^\n+/, '');
}

function parseMd(file) {
  const raw = fs.readFileSync(path.join(docsDir, file), 'utf-8');
  const text = stripFrontmatter(raw);
  const lines = text.split('\n');
  const h1 = lines.find((l) => l.startsWith('# ')) || '';
  const title = h1.replace(/^#\s+/, '').trim() || file;
  // summary = first non-empty paragraph after the h1 (skip quotes/code)
  let summary = '';
  let pastH1 = false;
  for (const line of lines) {
    const t = line.trim();
    if (!pastH1) {
      if (line.startsWith('# ')) pastH1 = true;
      continue;
    }
    if (!t || t.startsWith('>') || t.startsWith('```') || t.startsWith('#') || t.startsWith('|')) continue;
    summary += (summary ? ' ' : '') + t;
    if (summary.length > 220) break;
  }
  return { title, summary: summary.slice(0, 220), body: text.trim() + '\n' };
}

const out = PAGES.map(([file, slug, section, order]) => {
  const { title, summary, body } = parseMd(file);
  return { title, slug, section, sectionLabel: SECTION_LABEL[section], order, summary, body };
});

const seedDir = path.join(cmsDir, 'seed');
fs.mkdirSync(seedDir, { recursive: true });
fs.writeFileSync(path.join(seedDir, 'docs.json'), JSON.stringify(out, null, 2) + '\n');
console.log(`wrote seed/docs.json — ${out.length} pages`);
