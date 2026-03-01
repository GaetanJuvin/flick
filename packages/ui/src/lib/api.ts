const API_BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3456/api/v1';

export async function api<T>(
  path: string,
  opts: RequestInit & { cookie?: string } = {},
): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(opts.headers as Record<string, string> ?? {}),
  };
  if (opts.cookie) {
    headers['Cookie'] = opts.cookie;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...opts,
    headers,
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body?.error?.message ?? `API error: ${res.status}`);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}
