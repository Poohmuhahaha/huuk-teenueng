import { z } from 'zod'
import type { ApiErrorResponse } from '../types/ApiErrorResponse'
import type { Brand } from '../types/Brand'
import type { Campaign } from '../types/Campaign'
import type { Content } from '../types/Content'
import type { ContentRevision } from '../types/ContentRevision'
import type { ContentSummary } from '../types/ContentSummary'
import type { FontAsset } from '../types/FontAsset'
import type { HashtagGroup } from '../types/HashtagGroup'
import type { Idea } from '../types/Idea'
import type { Metric } from '../types/Metric'
import type { PlatformConnection } from '../types/PlatformConnection'
import type { Post } from '../types/Post'
import type { Role } from '../types/Role'
import type { SetupConfig } from '../types/SetupConfig'
import type { Txn } from '../types/Txn'
import type { User } from '../types/User'
import type { WorkspaceSummary } from '../types/WorkspaceSummary'

// Compile-time type equality check per Master Engineering SOP §1.2.
export type AssertEqual<T, Expected> = [T] extends [Expected]
  ? [Expected] extends [T]
    ? true
    : false
  : false

export const ApiErrorResponseSchema = z.object({
  error_code: z.string(),
  message: z.string(),
  details: z.record(z.string(), z.unknown()).nullable(),
  error: z.string(),
})
const _checkApiErrorResponse: AssertEqual<z.infer<typeof ApiErrorResponseSchema>, ApiErrorResponse> = true
void _checkApiErrorResponse

export const UserSchema = z.object({
  name: z.string(),
  role: z.string(),
})
const _checkUser: AssertEqual<z.infer<typeof UserSchema>, User> = true
void _checkUser

export const RoleSchema = z.object({
  name: z.string(),
  permissions: z.array(z.string()),
})
const _checkRole: AssertEqual<z.infer<typeof RoleSchema>, Role> = true
void _checkRole

export const SetupConfigSchema = z.object({
  language: z.string(),
  year: z.number(),
  owner: z.string(),
  workspaceName: z.string(),
  pillars: z.array(z.string()),
  formats: z.array(z.string()),
  goals: z.array(z.string()),
  statuses: z.array(z.string()),
  platforms: z.array(z.string()),
  showEditableColors: z.boolean(),
  users: z.array(UserSchema),
  authRequired: z.boolean(),
  allowRegistration: z.boolean(),
  roles: z.array(RoleSchema),
})
const _checkSetupConfig: AssertEqual<z.infer<typeof SetupConfigSchema>, SetupConfig> = true
void _checkSetupConfig

export const PostSchema = z.object({
  id: z.string(),
  month: z.number(),
  year: z.number(),
  topic: z.string(),
  pillar: z.string(),
  format: z.string(),
  goal: z.string(),
  date: z.string().nullable(),
  time: z.string(),
  status: z.string(),
  hook: z.string(),
  caption: z.string(),
  cta: z.string(),
  hashtagGroup: z.string(),
  hashtags: z.array(z.string()),
  imageUrl: z.string(),
  note: z.string(),
  done: z.boolean(),
  platforms: z.array(z.string()),
  lockedBy: z.string().nullable(),
})
const _checkPost: AssertEqual<z.infer<typeof PostSchema>, Post> = true
void _checkPost

export const IdeaSchema = z.object({
  id: z.string(),
  topic: z.string(),
  format: z.string(),
  idea: z.string(),
  link: z.string(),
  done: z.boolean(),
})
const _checkIdea: AssertEqual<z.infer<typeof IdeaSchema>, Idea> = true
void _checkIdea

export const HashtagGroupSchema = z.object({
  id: z.string(),
  title: z.string(),
  tags: z.array(z.string()),
})
const _checkHashtagGroup: AssertEqual<z.infer<typeof HashtagGroupSchema>, HashtagGroup> = true
void _checkHashtagGroup

export const MetricSchema = z.object({
  postId: z.string(),
  platform: z.string(),
  likes: z.number(),
  views: z.number(),
})
const _checkMetric: AssertEqual<z.infer<typeof MetricSchema>, Metric> = true
void _checkMetric

export const TxnSchema = z.object({
  id: z.string(),
  date: z.string(),
  amount: z.number(),
  kind: z.string(),
  category: z.string(),
  sub: z.string(),
})
const _checkTxn: AssertEqual<z.infer<typeof TxnSchema>, Txn> = true
void _checkTxn

export const BrandSchema = z.object({
  channel: z.string(),
  positioning: z.string(),
  slogan: z.string(),
  audience: z.string(),
  voice: z.string(),
  dos: z.array(z.string()),
  donts: z.array(z.string()),
  palette: z.array(z.string()),
  fonts: z.array(z.string()),
  logos: z.array(z.string()),
  moodboard: z.array(z.string()),
  radius: z.number(),
  fillOpacity: z.number(),
  strokeWidth: z.number(),
  shadow: z.string(),
})
const _checkBrand: AssertEqual<z.infer<typeof BrandSchema>, Brand> = true
void _checkBrand

export const FontAssetSchema = z.object({
  name: z.string(),
  family: z.string(),
  url: z.string(),
  size: z.number(),
  uploadedAt: z.number(),
})
const _checkFontAsset: AssertEqual<z.infer<typeof FontAssetSchema>, FontAsset> = true
void _checkFontAsset

export const PlatformConnectionSchema = z.object({
  id: z.string(),
  status: z.string(),
  handle: z.string(),
  externalId: z.string(),
  scopes: z.array(z.string()),
  tokenType: z.string(),
  expiresAt: z.string().nullable(),
  lastSync: z.string().nullable(),
  mediaCount: z.number(),
  note: z.string(),
})
const _checkPlatformConnection: AssertEqual<z.infer<typeof PlatformConnectionSchema>, PlatformConnection> = true
void _checkPlatformConnection

export const WorkspaceSummarySchema = z.object({
  id: z.string(),
  name: z.string(),
  created: z.string(),
  connected: z.number(),
  total: z.number(),
  isOwner: z.boolean(),
})
const _checkWorkspaceSummary: AssertEqual<z.infer<typeof WorkspaceSummarySchema>, WorkspaceSummary> = true
void _checkWorkspaceSummary

export const CampaignSchema = z.object({
  id: z.string(),
  name: z.string(),
  objective: z.string(),
  status: z.string(),
  startDate: z.string().nullable(),
  endDate: z.string().nullable(),
  platforms: z.array(z.string()),
  pillars: z.array(z.string()),
  hashtags: z.array(z.string()),
  budget: z.number(),
  goalMetric: z.string(),
  goalTarget: z.number(),
  owner: z.string(),
  notes: z.string(),
  contentIds: z.array(z.string()),
  createdAt: z.string(),
  updatedAt: z.string(),
})
const _checkCampaign: AssertEqual<z.infer<typeof CampaignSchema>, Campaign> = true
void _checkCampaign

export const ContentRevisionSchema = z.object({
  revision: z.number(),
  savedAt: z.string(),
  author: z.string(),
  note: z.string(),
  title: z.string(),
  body: z.string(),
  excerpt: z.string(),
  seoTitle: z.string(),
  seoDescription: z.string(),
  tags: z.array(z.string()),
  status: z.string(),
})
const _checkContentRevision: AssertEqual<z.infer<typeof ContentRevisionSchema>, ContentRevision> = true
void _checkContentRevision

export const ContentSchema = z.object({
  id: z.string(),
  title: z.string(),
  slug: z.string(),
  kind: z.string(),
  status: z.string(),
  body: z.string(),
  excerpt: z.string(),
  heroImageUrl: z.string(),
  tags: z.array(z.string()),
  seoTitle: z.string(),
  seoDescription: z.string(),
  author: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
  publishedAt: z.string().nullable(),
  scheduledFor: z.string().nullable(),
  version: z.number(),
  revisions: z.array(ContentRevisionSchema),
})
const _checkContent: AssertEqual<z.infer<typeof ContentSchema>, Content> = true
void _checkContent

export const ContentSummarySchema = z.object({
  id: z.string(),
  title: z.string(),
  slug: z.string(),
  kind: z.string(),
  status: z.string(),
  excerpt: z.string(),
  heroImageUrl: z.string(),
  tags: z.array(z.string()),
  author: z.string(),
  updatedAt: z.string(),
  publishedAt: z.string().nullable(),
  scheduledFor: z.string().nullable(),
  version: z.number(),
  revisionCount: z.number(),
  wordCount: z.number(),
})
const _checkContentSummary: AssertEqual<z.infer<typeof ContentSummarySchema>, ContentSummary> = true
void _checkContentSummary
