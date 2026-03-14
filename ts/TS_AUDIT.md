# TypeScript / React / Next.js Code Quality Audit

## Purpose

Audit the EVMAuth managed service platform TypeScript frontend for
correctness, type safety, security, and architectural adherence. The bar is
production-grade Next.js code written by a senior engineer. Flag anything
that would cause a strong TypeScript or React engineer to lose confidence in
the codebase.

The frontend is a **dumb interface**: it renders data, collects input, and
delegates all logic to the backend via the API proxy. Any business logic,
authorization decisions, or data transformation beyond display concerns found
in the frontend is an architectural violation and must be flagged as Critical.

Work through every item in every section below. For each finding provide:
- The file path and line range
- A concise description of the problem
- A concrete corrected example

At the end, produce a prioritized summary: **Critical** (correctness,
security, or architectural violation), **Major** (type safety or
maintainability), **Minor** (style or polish).

---

## Section 1: TypeScript Strictness

### 1.1 No `any`

`any` defeats the type system entirely. Flag every use of `any` in non-test
code, including:
- Explicit `any` annotations
- Implicit `any` from untyped function parameters
- `as any` casts
- `// @ts-ignore` and `// @ts-expect-error` without a specific explanation
  comment on the same line

The only acceptable use of `as` casts is narrowing from a known broader type
to a known narrower type where TypeScript cannot infer it (e.g. casting a
`Response` body after JSON parse against a known type). Even then, prefer a
Zod schema or type guard over a cast.

### 1.2 `tsconfig` strict mode is enabled

Every `tsconfig.json` in the workspace must have `"strict": true`. Flag any
that do not. Specifically verify these compiler options are active (enabled by
`strict`, but call out if overridden):
- `strictNullChecks`
- `noImplicitAny`
- `strictFunctionTypes`
- `strictPropertyInitialization`

Also flag if `"skipLibCheck": true` is set — this silently ignores type errors
in dependencies and can mask real problems. It is acceptable only as a
temporary workaround with a `// TODO` comment explaining why.

### 1.3 No non-null assertions without justification

The `!` non-null assertion operator suppresses a null check. Flag every `!`
postfix operator unless it is accompanied by a comment explaining why the
value is guaranteed non-null at that point. Prefer explicit null checks or
optional chaining.

### 1.4 Return types are explicit on exported functions

Exported functions and React components must have explicit return type
annotations. TypeScript can infer them, but explicit types catch unintentional
return type changes at the call site rather than at the implementation.

Flag any exported function or component lacking an explicit return type.

### 1.5 Enums are `const enum` or replaced with `as const` objects

TypeScript `enum` compiles to a runtime object, which adds bundle weight and
behaves unexpectedly with some bundlers. Prefer:
```typescript
// Instead of:
enum Role { Owner = 'owner', Admin = 'admin' }

// Use:
const Role = { Owner: 'owner', Admin: 'admin' } as const;
type Role = typeof Role[keyof typeof Role];
```
Flag all non-`const` enum declarations.

---

## Section 2: React and Next.js Correctness

### 2.1 Server Components do not import client-only code

Next.js App Router Server Components must not import:
- React hooks (`useState`, `useEffect`, etc.)
- Browser APIs (`window`, `document`, `localStorage`)
- `@turnkey/sdk-browser` or any client-side Turnkey SDK
- `iron-session` session reading (use the server-side `getIronSession` in
  Server Components, not the client session hook)

Flag any Server Component (a file without `'use client'` at the top) that
imports any of the above.

### 2.2 Client Components are as small as possible

`'use client'` boundaries should be pushed as far down the component tree as
possible. A large layout or page marked `'use client'` just to use one
`useState` is a missed optimization — the interactive part should be extracted
into a small client component.

Flag any file with `'use client'` that:
- Contains more than one logical interactive concern
- Could have its interactive parts extracted into a child component while the
  parent remains a Server Component

### 2.3 `useEffect` is not used for data fetching

Data fetching in this codebase uses SWR hooks (`useMe`, `useOrgs`, etc.).
`useEffect` + `fetch` for data fetching is the old pattern and should not
appear. Flag any `useEffect` that contains a `fetch` call or sets state from
an async operation.

### 2.4 Keys in lists are stable and unique

React list keys must be stable identifiers from the data (e.g. `id` from an
API response), never array indices. Flag any `.map((item, index) =>` where
`index` is used as the `key` prop.

### 2.5 `next/image` is used for all images

The Next.js `<Image>` component handles optimization, lazy loading, and CLS
prevention. Flag any `<img>` tag in non-test code.

### 2.6 `next/link` is used for all internal navigation

Flag any `<a href="...">` pointing to an internal route. All internal
navigation must use `<Link href="...">` from `next/link`.

### 2.7 No direct `window.location` manipulation

Navigation must use `useRouter()` from `next/navigation`. Direct
`window.location.href` assignment or `window.location.replace()` calls
bypass Next.js's client-side routing. Flag them.

### 2.8 Loading and error states are handled for every SWR hook

Every component that calls a SWR hook must handle three states: loading, error,
and success. Flag any component that calls `useMe`, `useOrgs`, or any SWR hook
without rendering a loading state and an error state. Mantine's `<Skeleton>`
and `<Alert>` are available for this purpose.

### 2.9 Mantine form validation is used, not manual state

Forms must use `useForm` from `@mantine/form`. Flag any form that manages
field values and validation errors with `useState` manually instead.

---

## Section 3: API Proxy and Data Fetching

### 3.1 All API calls go through `/api/proxy`

The frontend must never call the backend URL directly. Every `fetch` call
in component code or SWR hooks must target `/api/proxy/...`. Flag any
`fetch` call targeting `process.env.BACKEND_URL` or any other direct backend
URL from client-side code.

### 3.2 The proxy route forwards the session cookie correctly

In `src/app/api/proxy/[...path]/route.ts`:
- The session must be read server-side using `getIronSession`
- The `Authorization` header or equivalent session credential must be attached
  to the forwarded request
- The original request's `Cookie` header must be stripped before forwarding
  (never forward raw browser cookies to the backend)
- The proxy must handle all HTTP methods: GET, POST, PATCH, DELETE
- Non-2xx responses from the backend must be forwarded to the client with
  their original status code, not swallowed as 500s

Flag any of these missing behaviors.

### 3.3 SWR hooks are defined in `lib/hooks.ts`, not inline

SWR `useSWR` calls must not appear directly in component files. They must be
wrapped in named hooks in `lib/hooks.ts` (e.g. `useMe`, `useOrgs`). This
centralizes the cache key definitions and makes revalidation predictable.
Flag any `useSWR(...)` call outside of `lib/hooks.ts`.

### 3.4 Mutation responses trigger revalidation

After a POST, PATCH, or DELETE via the API client, the relevant SWR cache
keys must be revalidated using `mutate`. Flag any write operation that does
not call `mutate` to refresh affected data.

### 3.5 API response types are validated, not assumed

Data from the API proxy arrives as `unknown`. It must be cast to a known type
using either:
- A type assertion against a type defined in `types/api.ts` with a comment
  acknowledging the cast is trusted (acceptable for v1)
- A Zod schema parse (preferred for security-sensitive responses like session
  data and auth responses)

Flag any API response that is used as a typed value without either of the
above. Specifically flag any response from auth endpoints (`/api/auth/*`) that
is not validated before use.

---

## Section 4: Session and Authentication

### 4.1 `SessionData` contains no sensitive values

`lib/session.ts` defines the `SessionData` type. It must contain only:
- `userId: string`
- `email: string`
- `displayName?: string`

Flag any sensitive value in `SessionData`: JWTs, Turnkey credentials, wallet
private keys, API keys, or authorization data. The session cookie is
encrypted by iron-session but the principle is defense in depth.

### 4.2 Session cookie options are correctly hardened

In `lib/session.ts`, the `sessionOptions` passed to `getIronSession` must
include:
```typescript
cookieName: 'evmauth_session',
password: process.env.SESSION_SECRET,  // 32+ chars, from env only
cookieOptions: {
  httpOnly: true,
  secure: process.env.NODE_ENV === 'production',
  sameSite: 'strict',
  maxAge: undefined,  // session cookie (expires on browser close)
                      // OR explicit maxAge matching JWT expiry
}
```
Flag any deviation: `sameSite: 'lax'` or weaker, `httpOnly: false`,
`secure: false` unconditionally, or `SESSION_SECRET` hardcoded as a string
literal.

### 4.3 The middleware protects all dashboard routes

`src/middleware.ts` must redirect unauthenticated requests to `/auth/login`
for all routes matching `/dashboard/:path*`. Flag if:
- The matcher pattern does not cover all dashboard sub-routes
- The middleware reads the session but does not redirect on missing/invalid
  session
- Auth routes (`/auth/*`) are accidentally included in the protected matcher

### 4.4 The auth callback does not expose tokens to the client

The OAuth callback route (`/api/auth/callback` or the page at
`/auth/callback`) receives an OIDC token or authorization code. This flow
must complete server-side. Flag any callback handler that:
- Returns a raw JWT or OIDC token in the response body to the browser
- Stores a JWT in `localStorage` or `sessionStorage`
- Sets a non-HttpOnly cookie containing an auth token

### 4.5 `SESSION_SECRET` comes from environment only

Flag any file that contains a hardcoded fallback for `SESSION_SECRET`:
```typescript
// WRONG — never do this:
password: process.env.SESSION_SECRET ?? 'fallback-secret-for-dev'
```
In development, the `.env` file must provide the secret. A hardcoded fallback
trains developers to run without a real secret and may leak into production.

---

## Section 5: Security

### 5.1 No secrets in client-side code

`NEXT_PUBLIC_*` environment variables are embedded in the client bundle and
visible to anyone. Flag any variable that should be server-side only but is
prefixed with `NEXT_PUBLIC_`:
- `SESSION_SECRET`
- `BACKEND_URL` (internal service URL)
- Any Turnkey API private key
- Any database URL

### 5.2 User-supplied values are not rendered as HTML

Flag any use of `dangerouslySetInnerHTML`. If it exists, it must be
accompanied by a comment proving the content is sanitized and explaining why
raw HTML is necessary.

### 5.3 Redirect targets are validated

In the end-user auth flow, the `redirect_uri` parameter must be validated
server-side against the registered `callback_urls` before the redirect is
issued. Flag any redirect in the auth flow that uses a URL from a query
parameter without validation. This is an open redirect vulnerability.

Flag specifically the `/auth/end-user/login` page and its associated API
route for this check.

### 5.4 CSRF protection is not bypassed

Next.js App Router API routes are same-origin by default. Flag any API route
that:
- Disables CSRF protection explicitly
- Accepts requests from arbitrary origins via a permissive CORS header
  (`Access-Control-Allow-Origin: *`) on a state-mutating endpoint

The proxy route must not add permissive CORS headers.

### 5.5 `console.log` is absent from production code

`console.log` in production code may leak sensitive data to browser devtools
and server logs. Flag every `console.log`, `console.debug`, and
`console.dir` in non-test code. Use `console.error` only for genuine error
reporting, and only with controlled data.

---

## Section 6: Component Architecture

### 6.1 The "dumb interface" principle is enforced

The frontend contains no business logic. Flag any component or utility that:
- Makes an authorization decision (e.g. checking if a user has a role to
  show/hide UI elements based on locally-held data rather than API response)
- Transforms or derives domain data beyond formatting for display (e.g.
  computing whether a contract is "active" from raw fields)
- Contains conditional logic that duplicates backend validation

The only acceptable client-side logic is: rendering, formatting, form
validation (for UX feedback only — not as a security gate), and navigation.

### 6.2 Components are single-responsibility

Flag any component file that:
- Exceeds 200 lines (a strong signal it does too much)
- Contains both data-fetching logic (SWR hooks) and complex render logic that
  could be separated into a container and a presentational component
- Defines more than one exported component

### 6.3 Props are typed with interfaces, not inline types

Component props must be defined as a named `interface` or `type` above the
component, not as an inline object type in the function signature.

```typescript
// WRONG:
export function OrgCard({ org, onClick }: { org: OrgResponse; onClick: () => void }) {}

// CORRECT:
interface OrgCardProps {
  org: OrgResponse;
  onClick: () => void;
}
export function OrgCard({ org, onClick }: OrgCardProps): JSX.Element {}
```

Flag all inline prop type definitions.

### 6.4 Shared types live in `types/api.ts`

Any type that represents a backend API response shape must be defined in
`types/api.ts`, not inline in a component or hook file. Flag response types
defined anywhere else.

### 6.5 The `ui` package is used for shared components

Any component that is used in more than one place within the `dashboard`
service, or that is a candidate for reuse across services, must live in
`packages/ui/src/components/`. Flag duplicated component definitions across
files within the service.

---

## Section 7: Biome Configuration and Code Style

### 7.1 Biome passes cleanly

Run:
```
pnpm --filter='*' exec biome check .
```
Zero errors are acceptable. Flag any suppression comment (`// biome-ignore`)
that lacks a specific explanation.

### 7.2 Imports are organized

Biome's `organizeImports` must be enabled in `biome.json`. Imports must be
ordered: external packages first, then internal `@evmauth/*` packages, then
relative imports. Flag any file where this order is violated.

### 7.3 No barrel re-exports that create circular dependencies

`packages/ui/src/index.ts` re-exports components. Flag any circular import
where a service imports from `@evmauth/ui` and `@evmauth/ui` in turn imports
from the service.

---

## Section 8: Next.js App Router Specifics

### 8.1 `loading.tsx` files exist for data-dependent routes

Any route segment that fetches data (or whose layout fetches data) must have
a `loading.tsx` file alongside it using Mantine's `<Skeleton>` or a spinner.
Flag any `page.tsx` that depends on async data without a sibling
`loading.tsx`.

### 8.2 `error.tsx` files exist for fallible routes

Any route that could throw (network failures, unexpected API shapes) must have
an `error.tsx` file. Flag route segments without one.

### 8.3 Metadata is defined for public pages

`layout.tsx` and `page.tsx` for public-facing pages (`/`, `/auth/login`,
`/auth/end-user/login`) must export a `metadata` object or `generateMetadata`
function. Flag public pages without metadata.

### 8.4 Environment variables are accessed through a validated config module

`process.env.*` must not be accessed directly in component files or hooks.
All environment variable access must go through a single config module (e.g.
`lib/config.ts`) that validates presence and throws a clear error at startup
if required variables are missing.

Flag any direct `process.env.*` access outside of a dedicated config module.

### 8.5 The `api/proxy` route does not log request bodies

The proxy route forwards requests to the backend. It must not log request or
response bodies, as these may contain session credentials or sensitive user
data. Flag any `console.log` or structured log statement in the proxy handler
that logs a full request or response body.

---

## Section 9: Workspace and Package Hygiene

### 9.1 `packages/ui` has no runtime dependencies on Next.js

`packages/ui` is a shared component library. It must not have `next` as a
dependency (it can be a `peerDependency` or `devDependency` if needed for
types). Flag any `import` from `next/*` in `packages/ui/src/`.

### 9.2 `packages/tsconfig` is extended, not duplicated

Every `tsconfig.json` in services and packages must extend from
`@evmauth/tsconfig/base.json` or `@evmauth/tsconfig/nextjs.json`. Flag any
`tsconfig.json` that re-declares compiler options already defined in the base
configs.

### 9.3 Dependencies are declared in the correct package

A dependency used only by `services/dashboard` must be declared in
`services/dashboard/package.json`, not in the workspace root
`package.json`. Flag any dependency in the root `package.json` that is only
consumed by one service or package.

### 9.4 `pnpm-lock.yaml` is committed and up to date

The lockfile must be committed. Flag if it is in `.gitignore`. Also flag if
`package.json` dependency versions and the lockfile are out of sync (run
`pnpm install --frozen-lockfile` to verify).

---

## How to Run This Audit

```bash
cd ts

# Type checking
pnpm --filter='*' exec tsc --noEmit

# Linting and formatting
pnpm --filter='*' exec biome check .

# Dependency audit
pnpm audit
```

All three must produce zero errors before the audit findings are addressed.

Then work through each section above file by file, in this order:

1. `services/dashboard/src/app/api/` — proxy and auth routes (security boundary)
2. `services/dashboard/src/middleware.ts` — session protection
3. `services/dashboard/src/lib/` — session, api-client, hooks
4. `services/dashboard/src/app/auth/` — auth flow pages
5. `services/dashboard/src/app/dashboard/` — dashboard pages
6. `services/dashboard/src/components/` — components
7. `services/dashboard/src/types/` — shared types
8. `packages/ui/src/` — shared UI package

Produce findings in this format:

```
[CRITICAL|MAJOR|MINOR] services/dashboard/src/<file>.tsx:<line>
Problem: <one sentence>
Current:
    <offending code>
Fixed:
    <corrected code>
```
