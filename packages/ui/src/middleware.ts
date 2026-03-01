import { defineMiddleware } from 'astro:middleware';

const PUBLIC_PATHS = ['/login', '/api/v1/auth/saml'];

export const onRequest = defineMiddleware(async (context, next) => {
  const { pathname } = context.url;

  // Allow public paths
  if (PUBLIC_PATHS.some((p) => pathname.startsWith(p))) {
    return next();
  }

  // Check session cookie
  const session = context.cookies.get('session')?.value;
  if (!session) {
    return context.redirect('/login');
  }

  try {
    const user = JSON.parse(Buffer.from(session, 'base64').toString('utf-8'));
    context.locals.user = user;
  } catch {
    return context.redirect('/login');
  }

  return next();
});
