# Stage 1: Build
FROM node:22-slim AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .

ARG PUBLIC_APP_NAME="Otherworlds RPG"
ARG API_BASE_URL="http://localhost:3000"
ENV PUBLIC_APP_NAME=${PUBLIC_APP_NAME}
ENV API_BASE_URL=${API_BASE_URL}

RUN npm run build

# Stage 2: Runtime
FROM node:22-slim AS runtime

WORKDIR /app

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 3000

ENV NODE_ENV=production
ENV PORT=3000
ENV HOST=0.0.0.0

CMD ["node", "build"]
