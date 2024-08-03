import { createServer } from "http";
import { parse } from "url";

const port = parseInt(process.env.PORT ?? '9243', 10);
const dev = process.env.NODE_ENV !== "production";

async function main() {
  console.log('Starting server...');

  console.log(
    `> Server listening at http://localhost:${port} as ${
      dev ? "development" : process.env.NODE_ENV
    }`,
  );
}

main()
