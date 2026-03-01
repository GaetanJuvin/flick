# Reliability

## Caching
- Redis write-through cache with TTL safety net
- SDKs cache full flag config in memory
- SDKs serve from cache during API outages

## SDK Resilience
- Exponential backoff on fetch failures
- Graceful degradation: stale cache > default values > error
- ETag-based polling to minimize bandwidth

## Database
- Connection pooling via `pg.Pool`
- Health check endpoints for PostgreSQL and Redis
- Migrations run sequentially with version tracking

## Monitoring
- Structured logging via Pino
- Request/response logging with correlation IDs
- Webhook delivery tracking with retry status
