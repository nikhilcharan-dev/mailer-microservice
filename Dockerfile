# base image
FROM node:20-slim

WORKDIR /app

COPY package.json package-lock.json ./

# RUN executes at build time
RUN npm ci

COPY . .

EXPOSE 3000

CMD ["npm", "start"]