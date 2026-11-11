FROM node:lts-alpine AS base

#
# Build env
#

FROM base AS builder

WORKDIR /app
ENV PATH=/app/node_modules/.bin:$PATH

COPY package.json ./
COPY package-lock.json ./
COPY tsconfig.json ./
RUN npm ci

ARG NEXT_PUBLIC_APP_NAME
ARG NEXT_PUBLIC_WEB_BASE_URL
ARG NEXT_PUBLIC_API_BASE_URL
ARG NEXT_PUBLIC_PBKDF2_ITERATIONS=600000
ARG NEXT_PUBLIC_PASSWORD_BYTES=32
ARG NEXT_PUBLIC_CHALLENGE_BYTES=32

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

# Next.js production server (opt-in).
FROM builder AS nextjs-builder
RUN HDROP_WEB_RUNTIME=nextjs npm run build

FROM base AS nextjs

WORKDIR /app
ENV NODE_ENV=production NEXT_TELEMETRY_DISABLED=1 HOSTNAME=0.0.0.0 PORT=80

COPY --from=nextjs-builder --chown=node:node /app/.next/standalone ./
COPY --from=nextjs-builder /app/public ./public
COPY --from=nextjs-builder /app/.next/static ./.next/static

USER node
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -q --spider http://127.0.0.1:80/ || exit 1
CMD ["node", "server.js"]

# Static frontend (default).
FROM builder AS static-builder
RUN npm run build

FROM nginx:stable-alpine AS static

RUN rm -rf /usr/share/nginx/html/*
COPY infra/nginx/nginx.conf /etc/nginx/nginx.conf
COPY --from=static-builder /app/out /usr/share/nginx/html

USER nginx

EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -q --spider http://127.0.0.1:80/ || exit 1

# The complete configuration needs no startup scripts or writable image paths.
ENTRYPOINT []
CMD ["nginx", "-g", "daemon off;"]
