const API_BASE = process.env.FLICK_API_URL ?? 'http://localhost:3000/api/v1';
const API_KEY = process.env.FLICK_API_KEY ?? '';

export async function flickApi<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...opts,
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${API_KEY}`,
      ...(opts.headers as Record<string, string> ?? {}),
    },
  });

  if (res.status === 204) return undefined as T;

  const body = await res.json();
  if (!res.ok) {
    throw new Error(body?.error?.message ?? `API error: ${res.status}`);
  }
  return body;
}
