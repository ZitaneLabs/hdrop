#!/usr/bin/env sh

# Migrate database
npx prisma migrate deploy

# Start server
pm2 start src/main.js --name api --attach