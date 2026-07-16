// Local example CRM fixture for the BYO HTTP portfolio demo. It requires the
// bearer token supplied through runx local credential delivery.
import { createServer } from "node:http";

const port = Number(process.env.PORT || 8734);
const expected = `Bearer ${process.env.EXAMPLE_CRM_TOKEN || "crm_demo_secret"}`;

const server = createServer((req, res) => {
  const url = new URL(req.url, `http://127.0.0.1:${port}`);
  if (req.headers.authorization !== expected) {
    res.writeHead(401, { "content-type": "application/json" });
    res.end(JSON.stringify({ error: "unauthorized" }));
    return;
  }
  const match = url.pathname.match(/^\/v1\/accounts\/([^/]+)$/);
  if (req.method === "GET" && match) {
    const id = decodeURIComponent(match[1]);
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify({ id, name: `account-${id}`, plan: "portfolio" }));
    return;
  }
  res.writeHead(404, { "content-type": "application/json" });
  res.end(JSON.stringify({ error: "not found" }));
});

server.listen(port, "127.0.0.1", () => {
  process.stdout.write(`example CRM fixture listening on ${port}\n`);
});
