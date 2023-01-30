#
# Build env
#

FROM node:19-alpine

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

# Install dependencies
COPY package.json ./
COPY yarn.lock ./
RUN yarn --frozen-lockfile

# Copy prisma schema and migrations
COPY prisma/schema.prisma ./prisma/schema.prisma
COPY prisma/migrations ./prisma/migrations
RUN npx prisma generate

# Copy source code
COPY src ./src

# Copy entrypoint script
COPY infra/entrypoint.sh ./
RUN chmod +x entrypoint.sh

# Install pm2
RUN npm i -g pm2

EXPOSE 80
ENTRYPOINT ["sh", "entrypoint.sh"]