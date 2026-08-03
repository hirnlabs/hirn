FROM node:22-alpine AS build
WORKDIR /app

# Install Bun
RUN npm install -g bun

COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

COPY . .
RUN bun run build

FROM nginx:1.27-alpine AS runtime
WORKDIR /usr/share/nginx/html

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/build ./

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
