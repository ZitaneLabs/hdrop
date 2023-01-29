#
# Build env
#

FROM node:19-alpine

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

COPY hausdrop-mock-server/package.json ./
COPY hausdrop-mock-server/yarn.lock ./
COPY hausdrop-mock-server/.env ./
RUN yarn --frozen-lockfile

COPY hausdrop-mock-server/ ./

EXPOSE 80
RUN yarn start