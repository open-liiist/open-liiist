FROM node:23

WORKDIR /app

COPY package*.json ./

RUN npm install && npm install typescript -g

COPY prisma ./prisma
RUN npx prisma generate

COPY . .

RUN tsc

EXPOSE 3002

CMD ["npm", "start"]
