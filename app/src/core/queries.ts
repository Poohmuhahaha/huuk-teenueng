// TanStack Query hooks over the mock API.
// Pages never touch the store directly — only these hooks.
import { computed, toRef, toValue, unref } from 'vue'
import type { ComputedRef, MaybeRef } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import api from '@/api'
import { applyBrandTheme } from './theme'
import { currentName, currentUser, isLoggedIn, updateProfile } from './auth'
import type { Post, Metric, Permission, Brand, FontAsset, PlatformId, Campaign } from '@/mock/db'
import type { BrandImageKind } from './images'
import type { ContentPatchInput } from '@/api/contract'

export const qk = {
  setup: ['setup'] as const,
  allPosts: ['posts'] as const,
  posts: (m: number) => ['posts', m] as const,
  ideas: ['ideas'] as const,
  tags: ['tags'] as const,
  metrics: ['metrics'] as const,
  txns: ['txns'] as const,
  brand: ['brand'] as const,
  fonts: ['brand', 'fonts'] as const,
  campaigns: ['campaigns'] as const,
  platforms: ['platforms'] as const,
  live: ['live'] as const,
  ads: ['ads'] as const,
  accounts: ['accounts'] as const,
  workspaces: ['workspaces'] as const,
  workspaceMembers: (id: string) => ['workspaces', id, 'members'] as const,
}

export function useSetup() {
  return useQuery({ queryKey: qk.setup, queryFn: api.getSetup })
}

export function useSaveSetup() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.saveSetup,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useAddOption() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { list: 'pillars' | 'formats' | 'goals' | 'statuses' | 'platforms'; item: string }) =>
      api.addOption(v.list, v.item),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useAddUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { name: string; role: string }) => api.addUser(v.name, v.role),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useRemoveUser() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => api.removeUser(name),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useAddRole() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { name: string; permissions?: string[] }) => api.addRole(v.name, v.permissions),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useRemoveRole() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => api.removeRole(name),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.setup }),
  })
}

export function useAccounts() {
  return useQuery({ queryKey: qk.accounts, queryFn: api.listAccounts })
}

export function useUpdateProfile() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => updateProfile(name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.accounts })
      qc.invalidateQueries({ queryKey: qk.setup })
    },
  })
}

export function useDeleteAccount() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (email: string) => api.deleteAccount(email),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.accounts }),
  })
}

// ---- permissions ----

export function usePermission(): {
  can: (perm: Permission) => boolean
  isLoggedIn: ComputedRef<boolean>
  authRequired: ComputedRef<boolean>
} {
  const { data: setup } = useSetup()
  const authRequired = computed(() => setup.value?.authRequired ?? false)
  const can = (perm: Permission): boolean => {
    const s = setup.value
    if (!s) return false
    if (!s.authRequired) return true
    const user = currentUser.value
    if (!user) return false
    const role = s.roles.find((r) => r.name === user.role)
    return Boolean(role?.permissions.includes(perm))
  }
  return { can, isLoggedIn, authRequired }
}

function monthOf(m: MaybeRef<number>): number {
  return unref(m)
}

export function usePosts(month: MaybeRef<number>) {
  const m = computed(() => monthOf(month))
  return useQuery({ queryKey: computed(() => qk.posts(m.value)), queryFn: () => api.listPosts(m.value) })
}

/** All posts across every month (shared by dashboard/performance views). */
export function useAllPosts() {
  return useQuery({ queryKey: qk.allPosts, queryFn: () => api.listPosts() })
}

export function useUpdatePost(_month: MaybeRef<number>) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; patch: Partial<Post> & { user?: string } }) => api.updatePost(v.id, v.patch),
    onSuccess: () => {
      // A patch can move a post across months, so refresh the whole prefix.
      qc.invalidateQueries({ queryKey: qk.allPosts })
      qc.invalidateQueries({ queryKey: qk.metrics })
    },
  })
}

// ---- post locking (US-014) ----

export function useLockPost(month: MaybeRef<number>) {
  const qc = useQueryClient()
  const m = computed(() => monthOf(month))
  return useMutation({
    mutationFn: (id: string) => api.lockPost(id, currentName.value ?? ''),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.posts(m.value) }),
  })
}

export function useUnlockPost(month: MaybeRef<number>) {
  const qc = useQueryClient()
  const m = computed(() => monthOf(month))
  return useMutation({
    mutationFn: (id: string) => api.unlockPost(id, currentName.value ?? ''),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.posts(m.value) }),
  })
}

export function useAddPost(_month: MaybeRef<number>) {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (draft: Omit<Post, 'id'>) => api.addPost(draft),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.allPosts }),
  })
}

export function useDeletePost(month: MaybeRef<number>) {
  const qc = useQueryClient()
  const m = toRef(month)
  return useMutation({
    mutationFn: api.deletePost,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.posts(m.value) })
      qc.invalidateQueries({ queryKey: qk.allPosts })
    },
  })
}

export function useIdeas() {
  return useQuery({ queryKey: qk.ideas, queryFn: api.listIdeas })
}

export function useAddIdea() {
  const qc = useQueryClient()
  return useMutation({ mutationFn: api.addIdea, onSuccess: () => qc.invalidateQueries({ queryKey: qk.ideas }) })
}

export function useToggleIdea() {
  const qc = useQueryClient()
  return useMutation({ mutationFn: api.toggleIdea, onSuccess: () => qc.invalidateQueries({ queryKey: qk.ideas }) })
}

export function usePromoteIdea(month: MaybeRef<number>) {
  const qc = useQueryClient()
  const m = computed(() => monthOf(month))
  return useMutation({
    mutationFn: (id: string) => api.promoteIdea(id, m.value),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.ideas })
      qc.invalidateQueries({ queryKey: qk.allPosts })
    },
  })
}

export function useTags() {
  return useQuery({ queryKey: qk.tags, queryFn: api.listTags })
}

export function useAddTag() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { groupId: string; tag: string }) => api.addTag(v.groupId, v.tag),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.tags }),
  })
}

export function useMetrics() {
  return useQuery({ queryKey: qk.metrics, queryFn: api.listMetrics })
}

export function useImportMetrics() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { platform: string; month: number }) => api.importMetrics(v.platform, v.month),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.metrics }),
  })
}

export function useTxns() {
  return useQuery({ queryKey: qk.txns, queryFn: api.listTxns })
}

export function useAddTxn() {
  const qc = useQueryClient()
  return useMutation({ mutationFn: api.addTxn, onSuccess: () => qc.invalidateQueries({ queryKey: qk.txns }) })
}

export function useBrand() {
  return useQuery({ queryKey: qk.brand, queryFn: api.getBrand })
}

export function useCampaigns() {
  return useQuery({ queryKey: qk.campaigns, queryFn: api.listCampaigns })
}

export function useCreateCampaign() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.createCampaign,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.campaigns }),
  })
}

export function useUpdateCampaign() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, patch }: { id: string; patch: Partial<Campaign> }) => api.updateCampaign(id, patch),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.campaigns }),
  })
}

export function useDeleteCampaign() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.deleteCampaign,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.campaigns }),
  })
}

export function useUploadBrandImage() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, data, kind }: { name: string; data: string; kind: BrandImageKind }) =>
      api.uploadBrandImage(name, data, kind),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.brand }),
  })
}

export function useBrandFonts() {
  return useQuery({ queryKey: qk.fonts, queryFn: api.listFonts, staleTime: 5 * 60_000 })
}

export function useUploadFont() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, data }: { name: string; data: string }) => api.uploadFont(name, data),
    onSuccess: (fonts) => {
      qc.setQueryData(qk.fonts, fonts)
      applyBrandTheme(qc.getQueryData<Brand>(qk.brand), fonts)
      void qc.invalidateQueries({ queryKey: qk.fonts })
    },
  })
}

export function useDeleteFont() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => api.deleteFont(name),
    onSuccess: (fonts) => {
      qc.setQueryData(qk.fonts, fonts)
      applyBrandTheme(qc.getQueryData<Brand>(qk.brand), fonts)
      void qc.invalidateQueries({ queryKey: qk.fonts })
    },
  })
}

export function useSaveBrand() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.saveBrand,
    onSuccess: (data) => {
      // Apply the saved CI right away (the site theme) and refresh the cache.
      qc.setQueryData(qk.brand, data)
      applyBrandTheme(data, qc.getQueryData<FontAsset[]>(qk.fonts) ?? [])
      void qc.invalidateQueries({ queryKey: qk.brand })
    },
  })
}

export interface DashStats {
  total: number
  posted: number
  pending: number
  wip: number
  views: number
  likes: number
  pillarCounts: Record<string, number>
  platformCounts: Record<string, number>
  statusCounts: Record<string, number>
  top5: { post: Post; views: number }[]
}

function viewsOf(mets: Metric[], postId: string): number {
  return mets.filter((m) => m.postId === postId).reduce((s, m) => s + m.views, 0)
}

export function useDashboard(month: MaybeRef<number>) {
  const postsQ = usePosts(month)
  const metricsQ = useMetrics()
  const stats = computed<DashStats | null>(() => {
    const rows = postsQ.data.value
    if (!rows) return null
    // Only metrics that belong to the selected month's posts.
    const ids = new Set(rows.map((p) => p.id))
    const mets = (metricsQ.data.value ?? []).filter((m) => ids.has(m.postId))
    const pillarCounts: Record<string, number> = {}
    const platformCounts: Record<string, number> = {}
    const statusCounts: Record<string, number> = {}
    let views = 0
    let likes = 0
    for (const m of mets) {
      views += m.views
      likes += m.likes
    }
    for (const p of rows) {
      pillarCounts[p.pillar] = (pillarCounts[p.pillar] ?? 0) + 1
      statusCounts[p.status] = (statusCounts[p.status] ?? 0) + 1
      for (const pl of p.platforms) platformCounts[pl] = (platformCounts[pl] ?? 0) + 1
    }
    const top5 = [...rows]
      .map((post) => ({ post, views: viewsOf(mets, post.id) }))
      .sort((a, b) => b.views - a.views)
      .slice(0, 5)
    return {
      total: rows.length,
      posted: rows.filter((p) => p.status === 'Done').length,
      pending: rows.filter((p) => p.status === 'Start').length,
      wip: rows.filter((p) => p.status === 'Design' || p.status === 'Dev').length,
      views, likes, pillarCounts, platformCounts, statusCounts, top5,
    }
  })
  return { postsQ, metricsQ, stats }
}

// ---- workspaces ----
export function useWorkspaces() {
  return useQuery({ queryKey: qk.workspaces, queryFn: api.listWorkspaces })
}

/** Invalidates everything workspace-scoped — used after switching or deleting. */
export function invalidateWorkspaceData(qc: ReturnType<typeof useQueryClient>): void {
  qc.invalidateQueries()
}

export function useCreateWorkspace() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.createWorkspace,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.workspaces }),
  })
}

export function useRenameWorkspace() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) => api.renameWorkspace(id, name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.workspaces })
      qc.invalidateQueries({ queryKey: qk.setup })
    },
  })
}

export function useDeleteWorkspace() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.deleteWorkspace,
    onSuccess: () => invalidateWorkspaceData(qc),
  })
}

// ---- workspace members (owner-only) ----

export function useWorkspaceMembers(id: MaybeRef<string>) {
  const value = computed(() => unref(id))
  return useQuery({
    queryKey: computed(() => qk.workspaceMembers(value.value)),
    queryFn: () => api.listWorkspaceMembers(value.value),
    enabled: computed(() => Boolean(value.value)),
  })
}

export function useAddWorkspaceMember() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; email: string }) => api.addWorkspaceMember(v.id, v.email),
    onSuccess: (_data, v) => {
      qc.invalidateQueries({ queryKey: qk.workspaceMembers(v.id) })
      qc.invalidateQueries({ queryKey: qk.workspaces })
    },
  })
}

export function useRemoveWorkspaceMember() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { id: string; email: string }) => api.removeWorkspaceMember(v.id, v.email),
    onSuccess: (_data, v) => {
      qc.invalidateQueries({ queryKey: qk.workspaceMembers(v.id) })
      qc.invalidateQueries({ queryKey: qk.workspaces })
    },
  })
}

// ---- platform connections ----
export function usePlatforms() {
  return useQuery({ queryKey: qk.platforms, queryFn: api.listPlatforms })
}

export function useConnectPlatform() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, handle }: { id: PlatformId; handle?: string }) => api.connectPlatform(id, handle),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.platforms }),
  })
}

export function useDisconnectPlatform() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.disconnectPlatform,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.platforms }),
  })
}

export function useSyncPlatform() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.syncPlatform,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.platforms })
      qc.invalidateQueries({ queryKey: qk.live })
      qc.invalidateQueries({ queryKey: qk.metrics })
    },
  })
}

/** Mirror of the connected platforms' real content (posts + engagement). */
export function useLive() {
  return useQuery({ queryKey: qk.live, queryFn: api.getLive })
}

// ---- Meta Ads mirror + management ----

/** Meta Ads mirror: campaigns, ad sets, ads, settings and last-30d insights. */
export function useAds() {
  return useQuery({ queryKey: qk.ads, queryFn: api.getAds })
}

export function useSyncAds() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.syncAds,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.ads }),
  })
}

export function useSetAdsManage() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.setAdsManage,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.ads }),
  })
}

export function useAdCampaignStatus() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, status }: { id: string; status: 'ACTIVE' | 'PAUSED' | 'ARCHIVED' }) =>
      api.setAdCampaignStatus(id, status),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.ads }),
  })
}

export function useAdCampaignBudget() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, dailyBudget, lifetimeBudget }: {
      id: string
      dailyBudget?: number
      lifetimeBudget?: number
    }) => api.setAdCampaignBudget(id, { dailyBudget, lifetimeBudget }),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.ads }),
  })
}

export function useDuplicateAdCampaign() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.duplicateAdCampaign,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.ads }),
  })
}

export function useCreateAdBoost() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: api.createAdBoost,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.ads })
      qc.invalidateQueries({ queryKey: qk.platforms })
    },
  })
}

export function useOAuthPending(platform: MaybeRef<PlatformId | ''>, pick: MaybeRef<string>) {
  return useQuery({
    queryKey: computed(() => ['oauth-pick', toValue(platform), toValue(pick)] as const),
    queryFn: () => api.oauthPending(toValue(platform) as PlatformId, toValue(pick)),
    enabled: computed(() => toValue(platform) !== '' && toValue(pick) !== ''),
    staleTime: 0,
    retry: false,
  })
}

export function useChooseOAuthPage() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ id, pick, external_id }: { id: PlatformId; pick: string; external_id: string }) =>
      api.chooseOAuthPage(id, pick, external_id),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.platforms }),
  })
}

// ---- CMS: content studio ----

function useContentInvalidate(): () => void {
  const qc = useQueryClient()
  return () => {
    qc.invalidateQueries({ queryKey: ['content'] })
    qc.invalidateQueries({ queryKey: ['public-content'] })
  }
}

export function useContentList(params: MaybeRef<{ status?: string; q?: string }>) {
  const p = computed(() => unref(params))
  return useQuery({
    queryKey: computed(() => ['content', 'list', p.value.status ?? '', p.value.q ?? ''] as const),
    queryFn: () => api.listContent(p.value),
  })
}

export function useContentItem(id: MaybeRef<string | null>) {
  const value = computed(() => unref(id))
  return useQuery({
    queryKey: computed(() => ['content', 'item', value.value ?? ''] as const),
    queryFn: () => api.getContent(value.value as string),
    enabled: computed(() => Boolean(value.value)),
  })
}

export function useContentRevisions(id: MaybeRef<string | null>) {
  const value = computed(() => unref(id))
  return useQuery({
    queryKey: computed(() => ['content', 'revisions', value.value ?? ''] as const),
    queryFn: () => api.listContentRevisions(value.value as string),
    enabled: computed(() => Boolean(value.value)),
  })
}

export function useCreateContent() {
  const invalidate = useContentInvalidate()
  return useMutation({ mutationFn: api.createContent, onSuccess: invalidate })
}

export function useUpdateContent() {
  const invalidate = useContentInvalidate()
  return useMutation({
    mutationFn: (v: { id: string; patch: ContentPatchInput }) => api.updateContent(v.id, v.patch),
    onSuccess: invalidate,
  })
}

export function usePublishContent() {
  const invalidate = useContentInvalidate()
  return useMutation({
    mutationFn: (v: { id: string; version: number }) => api.publishContent(v.id, v.version),
    onSuccess: invalidate,
  })
}

export function useUnpublishContent() {
  const invalidate = useContentInvalidate()
  return useMutation({
    mutationFn: (v: { id: string; version: number }) => api.unpublishContent(v.id, v.version),
    onSuccess: invalidate,
  })
}

export function useScheduleContent() {
  const invalidate = useContentInvalidate()
  return useMutation({
    mutationFn: (v: { id: string; version: number; scheduledFor: string }) =>
      api.scheduleContent(v.id, v.version, v.scheduledFor),
    onSuccess: invalidate,
  })
}

export function useDeleteContent() {
  const invalidate = useContentInvalidate()
  return useMutation({ mutationFn: api.deleteContent, onSuccess: invalidate })
}

export function useDuplicateContent() {
  const invalidate = useContentInvalidate()
  return useMutation({ mutationFn: api.duplicateContent, onSuccess: invalidate })
}

export function useRestoreContentRevision() {
  const invalidate = useContentInvalidate()
  return useMutation({
    mutationFn: (v: { id: string; revision: number; version: number }) =>
      api.restoreContentRevision(v.id, v.revision, v.version),
    onSuccess: invalidate,
  })
}

export function usePublicContentList() {
  return useQuery({ queryKey: ['public-content', 'list'], queryFn: api.listPublicContent })
}

export function usePublicContent(slug: MaybeRef<string>) {
  const value = computed(() => unref(slug))
  return useQuery({
    queryKey: computed(() => ['public-content', 'item', value.value] as const),
    queryFn: () => api.getPublicContent(value.value),
  })
}

// ---- admin onboarding ----

export function useCreateAccount() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { name: string; email: string; role: string; password?: string }) => api.createAccount(v),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.accounts })
      qc.invalidateQueries({ queryKey: qk.setup })
    },
  })
}
