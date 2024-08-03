import { createServer } from "http";
import { parse } from "url";

const dev = process.env.NODE_ENV !== "production";

async function main() {
  const port = 9243;

  console.log(
    `> Server listening at http://localhost:${port} as ${
      dev ? "development" : process.env.NODE_ENV
    }`,
  );
}

main()
