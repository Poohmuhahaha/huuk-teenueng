// Shared OAuth hand-off: ask the server to create the CSRF state, then send the
// browser to the provider's own login page. Used by the connection dialog and
// by the live mirror's "reconnect" action.
import api from '@/api'
import type { PlatformId } from './platforms'
import { t } from './i18n'

// Provider hosts this app will ever redirect to (defense against a compromised
// or misconfigured API response).
const ALLOWED_OAUTH_HOSTS = [
  'facebook.com',
  'www.facebook.com',
  'accounts.google.com',
  'www.tiktok.com',
]

export function safeAuthorizeUrl(raw: string): string | null {
  try {
    const url = new URL(raw)
    if (url.protocol !== 'https:') return null
    if (!ALLOWED_OAUTH_HOSTS.some((host) => url.hostname === host)) return null
    return url.toString()
  } catch {
    return null
  }
}

/**
 * Starts the OAuth flow for `platform` and redirects the browser on success.
 * Returns an error message when the provider is not configured or the URL is
 * unsafe; returns null when the browser is navigating away.
 */
export async function beginOAuth(platform: PlatformId): Promise<string | null> {
  try {
    const res = await api.startOauth(platform)
    if (res.mode === 'redirect' && res.url) {
      const target = safeAuthorizeUrl(res.url)
      if (!target) return t('slogin.badRedirect')
      window.location.href = target
      return null
    }
    return t('slogin.notConfigured')
  } catch (e) {
    return e instanceof Error ? e.message : String(e)
  }
}
