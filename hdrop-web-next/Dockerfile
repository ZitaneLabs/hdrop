FROM node:19-alpine AS base

#
# Build env
#

FROM base AS builder

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

COPY package.json ./
COPY tsconfig.json ./
COPY yarn.lock ./
RUN yarn --frozen-lockfile

ARG NEXT_PUBLIC_APP_NAME
ARG NEXT_PUBLIC_WEB_BASE_URL
ARG NEXT_PUBLIC_API_BASE_URL
ARG NEXT_PUBLIC_PBKDF2_ITERATIONS
ARG NEXT_PUBLIC_PASSWORD_BYTES
ARG NEXT_PUBLIC_CHALLENGE_BYTES

ENV NEXT_TELEMETRY_DISABLED=1

ENV NEXT_PUBLIC_APP_NAME=${NEXT_PUBLIC_APP_NAME}
ENV NEXT_PUBLIC_WEB_BASE_URL=${NEXT_PUBLIC_WEB_BASE_URL}
ENV NEXT_PUBLIC_API_BASE_URL=${NEXT_PUBLIC_API_BASE_URL}
ENV NEXT_PUBLIC_PBKDF2_ITERATIONS=${NEXT_PUBLIC_PBKDF2_ITERATIONS}
ENV NEXT_PUBLIC_PASSWORD_BYTES=${NEXT_PUBLIC_PASSWORD_BYTES}
ENV NEXT_PUBLIC_CHALLENGE_BYTES=${NEXT_PUBLIC_CHALLENGE_BYTES}

COPY public ./public
COPY src ./src
COPY next.config.js ./
COPY postcss.config.js ./
COPY tailwind.config.js ./
RUN yarn build

#
# Production env
#

FROM base AS runner

WORKDIR /app

ENV NODE_ENV=production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 80
ENV PORT 80
CMD ["node", "server.js"]
