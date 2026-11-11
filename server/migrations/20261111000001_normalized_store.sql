-- Normalized store (Phase 1).
--
-- The legacy tables from the initial schema were never queried by the app and
-- used BIGINT ids. They are renamed instead of dropped so any data an operator
-- inserted manually survives. The new tables use the app's TEXT ids and keep
-- the full entity document in `payload` while promoting the columns we query
-- and index to real columns.

ALTER TABLE IF EXISTS workspace_members RENAME TO legacy_workspace_members;
ALTER TABLE IF EXISTS sessions RENAME TO legacy_sessions;
ALTER TABLE IF EXISTS campaigns RENAME TO legacy_campaigns;
ALTER TABLE IF EXISTS ideas RENAME TO legacy_ideas;
ALTER TABLE IF EXISTS finance_items RENAME TO legacy_finance_items;
ALTER TABLE IF EXISTS users RENAME TO legacy_users;
ALTER TABLE IF EXISTS workspaces RENAME TO legacy_workspaces;

-- Global settings: setup config + token salt + normalized schema marker.
CREATE TABLE app_settings (
    id SMALLINT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    email TEXT PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT '',
    plan TEXT NOT NULL DEFAULT 'free',
    status TEXT NOT NULL DEFAULT 'active',
    pw_hash TEXT NOT NULL DEFAULT '',
    last_active TEXT,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    owner_email TEXT NOT NULL DEFAULT '',
    created TEXT NOT NULL DEFAULT '',
    -- Preserves the snapshot's workspace order (the first one is the fallback).
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workspaces_owner ON workspaces (owner_email);
CREATE INDEX idx_workspaces_position ON workspaces (position);

CREATE TABLE workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',
    PRIMARY KEY (workspace_id, email)
);

CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_email ON sessions (email);
CREATE INDEX idx_sessions_expires ON sessions (expires_at);

CREATE TABLE posts (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    month INT NOT NULL DEFAULT 0,
    year INT NOT NULL DEFAULT 0,
    date TEXT,
    status TEXT NOT NULL DEFAULT '',
    platforms JSONB NOT NULL DEFAULT '[]'::jsonb,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_posts_ws_date ON posts (workspace_id, year, month, date);
CREATE INDEX idx_posts_ws_status ON posts (workspace_id, status);

CREATE TABLE post_platforms (
    workspace_id TEXT NOT NULL,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    PRIMARY KEY (workspace_id, post_id, platform),
    FOREIGN KEY (workspace_id, post_id) REFERENCES posts (workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE ideas (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    topic TEXT NOT NULL DEFAULT '',
    format TEXT NOT NULL DEFAULT '',
    done BOOLEAN NOT NULL DEFAULT FALSE,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_ideas_ws_done ON ideas (workspace_id, done);

CREATE TABLE hashtag_groups (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE TABLE metrics (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    likes BIGINT NOT NULL DEFAULT 0,
    views BIGINT NOT NULL DEFAULT 0,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, post_id, platform)
);

CREATE INDEX idx_metrics_ws_post ON metrics (workspace_id, post_id);

CREATE TABLE finance_entries (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    date TEXT NOT NULL DEFAULT '',
    amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    kind TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT '',
    sub TEXT NOT NULL DEFAULT '',
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_finance_ws_date ON finance_entries (workspace_id, date);

CREATE TABLE content_items (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    slug TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL DEFAULT 'article',
    status TEXT NOT NULL DEFAULT 'draft',
    author TEXT NOT NULL DEFAULT '',
    updated_at_text TEXT NOT NULL DEFAULT '',
    published_at TEXT,
    scheduled_for TEXT,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_content_ws_status ON content_items (workspace_id, status);
CREATE INDEX idx_content_ws_slug ON content_items (workspace_id, slug);

CREATE TABLE content_revisions (
    workspace_id TEXT NOT NULL,
    content_id TEXT NOT NULL,
    revision INT NOT NULL,
    saved_at TEXT NOT NULL DEFAULT '',
    author TEXT NOT NULL DEFAULT '',
    payload JSONB NOT NULL,
    PRIMARY KEY (workspace_id, content_id, revision),
    FOREIGN KEY (workspace_id, content_id) REFERENCES content_items (workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE campaigns (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    objective TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'draft',
    start_date TEXT,
    end_date TEXT,
    budget DOUBLE PRECISION NOT NULL DEFAULT 0,
    position INT NOT NULL DEFAULT 0,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_campaigns_ws_status ON campaigns (workspace_id, status);

CREATE TABLE campaign_content (
    workspace_id TEXT NOT NULL,
    campaign_id TEXT NOT NULL,
    content_id TEXT NOT NULL,
    PRIMARY KEY (workspace_id, campaign_id, content_id),
    FOREIGN KEY (workspace_id, campaign_id) REFERENCES campaigns (workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE platform_connections (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'disconnected',
    handle TEXT NOT NULL DEFAULT '',
    external_id TEXT NOT NULL DEFAULT '',
    expires_at TEXT,
    payload JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE TABLE platform_tokens (
    workspace_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    sealed TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, platform)
);

CREATE TABLE media_assets (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT '',
    r2_key TEXT NOT NULL UNIQUE,
    mime TEXT NOT NULL DEFAULT '',
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT NOT NULL DEFAULT '',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, id)
);

CREATE INDEX idx_media_ws_kind ON media_assets (workspace_id, kind);

CREATE TABLE outbox_events (
    id BIGSERIAL PRIMARY KEY,
    workspace_id TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL,
    entity_id TEXT NOT NULL DEFAULT '',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

CREATE INDEX idx_outbox_unpublished ON outbox_events (created_at) WHERE published_at IS NULL;

CREATE TABLE store_meta (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
