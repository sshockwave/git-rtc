import { createServer } from './server';

const dev = process.env.NODE_ENV !== "production";

async function main() {
  createServer().listen({
    host: '127.0.0.1',
    port: 9242,
  }, (err, address) => {
    if (err) {
      console.error(err);
      process.exit(1);
    }
    console.log(`Server listening at ${address}`);
  });
}

main()
