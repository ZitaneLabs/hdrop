#
# Build env
#

FROM node:19-alpine

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

COPY package.json ./
COPY yarn.lock ./
RUN yarn --frozen-lockfile
RUN npm i -g pm2

COPY . ./

EXPOSE 80
CMD ["pm2", "start", "src/main.js", "--name", "api", "--attach"]