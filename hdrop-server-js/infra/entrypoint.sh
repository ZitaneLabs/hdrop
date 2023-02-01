#!/usr/bin/env sh

# Migrate database
npx prisma migrate deploy

# Build server
yarn build

# Start server
pm2-runtime start dist/main.js