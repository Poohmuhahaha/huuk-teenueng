const KEY = 'huuk.login.redirect'

function isAppPath(path: string): boolean {
  return (
    path.startsWith('/') &&
    !path.startsWith('//') &&
    path !== '/login' &&
    !path.startsWith('/login/') &&
    path !== '/register' &&
    !path.startsWith('/register/')
  )
}

export function rememberLoginRedirect(path: string): void {
  if (!isAppPath(path)) return
  try {
    sessionStorage.setItem(KEY, path)
  } catch {
    return
  }
}

export function takeLoginRedirect(): string {
  try {
    const path = sessionStorage.getItem(KEY)
    sessionStorage.removeItem(KEY)
    if (path && isAppPath(path)) return path
  } catch {
    return '/'
  }
  return '/'
}
