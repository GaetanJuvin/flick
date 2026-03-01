import type { APIRoute } from 'astro';

const API_BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:3456/api/v1';

export const ALL: APIRoute = async ({ params, request, cookies }) => {
  const path = params.path;
  const sessionCookie = cookies.get('session')?.value;

  if (!sessionCookie) {
    return new Response(JSON.stringify({ error: { code: 'UNAUTHORIZED', message: 'Not authenticated' } }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const headers: Record<string, string> = {
    'Cookie': `session=${sessionCookie}`,
  };

  // Forward content-type and body for non-GET requests
  const contentType = request.headers.get('content-type');
  if (contentType) {
    headers['Content-Type'] = contentType;
  }

  const fetchOpts: RequestInit = {
    method: request.method,
    headers,
  };

  if (request.method !== 'GET' && request.method !== 'HEAD') {
    fetchOpts.body = await request.text();
  }

  const res = await fetch(`${API_BASE}/${path}`, fetchOpts);
  const body = await res.text();

  return new Response(body, {
    status: res.status,
    headers: { 'Content-Type': res.headers.get('content-type') ?? 'application/json' },
  });
};
