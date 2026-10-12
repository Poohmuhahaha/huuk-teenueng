import fs from 'fs';
import path from 'path';
import type { Core } from '@strapi/strapi';

interface SeedDoc {
  title: string;
  slug: string;
  section: 'start' | 'workflows' | 'authoring' | 'reference' | 'templates' | 'examples';
  sectionLabel: string;
  order: number;
  summary: string;
  body: string;
}

// Every boot: ensure docs/*.md seed (committed as seed/docs.json) exists,
// and open read-only API (find/findOne) to the Public role so docs-web
// can fetch live without a token.
async function bootstrap({ strapi }: { strapi: Core.Strapi }) {
  const uid = 'api::doc-page.doc-page' as const;

  // Idempotent: safe to run every boot (skips slugs editors already own).
  {
    const seedPath = path.join(process.cwd(), 'seed', 'docs.json');
    if (fs.existsSync(seedPath)) {
      const docs = JSON.parse(fs.readFileSync(seedPath, 'utf-8')) as SeedDoc[];
      let created = 0;
      for (const doc of docs) {
        // Skip slugs an editor already owns (draft or published) — never
        // overwrite manual CMS edits; publish leftover draft seeds instead.
        const draft = await strapi.documents(uid).findFirst({
          filters: { slug: doc.slug },
          status: 'draft',
        });
        const live = draft ??
          (await strapi.documents(uid).findFirst({ filters: { slug: doc.slug } }));
        if (!live) {
          // NOTE (v5): publishing = top-level `status`, not a publishedAt field.
          await strapi.documents(uid).create({ data: { ...doc }, status: 'published' });
          created += 1;
        } else if (!live.publishedAt) {
          await strapi.documents(uid).publish({ documentId: live.documentId });
          created += 1;
        }
      }
      strapi.log.info(`[bootstrap] seed ensured (${created} created/published, ${docs.length} total)`);
    } else {
      strapi.log.warn('[bootstrap] seed/docs.json not found — skipping seed');
    }
  }

  try {
    // Same mechanism as saving a role in admin (Settings → Roles):
    // read the full permission layout, flip our two actions on,
    // write it back — the service creates the missing rows.
    // NOTE: documents/query APIs can't do this (no `enabled` column in v5).
    const roleService = strapi.plugin('users-permissions').service('role');
    const publicRole = await strapi.db.query('plugin::users-permissions.role').findOne({
      where: { type: 'public' },
    });
    if (publicRole) {
      const full = await roleService.findOne(publicRole.id);
      const ctrl = full?.permissions?.['api::doc-page']?.controllers?.['doc-page'];
      if (ctrl?.find && ctrl?.findOne) {
        ctrl.find.enabled = true;
        ctrl.findOne.enabled = true;
        await roleService.updateRole(publicRole.id, { permissions: full.permissions });
        strapi.log.info('[bootstrap] public find/findOne enabled for doc-page');
      } else {
        throw new Error('doc-page actions missing from permission layout');
      }
    }
  } catch (err) {
    strapi.log.warn(`[bootstrap] public perms failed: ${(err as Error).message}`);
    strapi.log.warn(
      '[bootstrap] open Settings → Roles → Public and tick doc-page find/findOne manually',
    );
  }
}

export default {
  register() {},
  bootstrap,
};
