#
# Build env
#

FROM node:19-alpine

WORKDIR /app
ENV PATH /app/node_modules/.bin:$PATH

COPY package.json ./
COPY yarn.lock ./
RUN yarn --frozen-lockfile

ARG REACT_APP_BASE_URL
ARG REACT_APP_API_ENDPOINT
ENV REACT_APP_BASE_URL=${REACT_APP_BASE_URL}
ENV REACT_APP_API_ENDPOINT=${REACT_APP_API_ENDPOINT}

COPY public ./public
COPY src ./src
RUN yarn build

#
# Production env
#

FROM nginx:alpine

COPY --from=0 /app/build /usr/share/nginx/html
COPY infra/nginx/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]