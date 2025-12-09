// website/src/index.ts
import { Elysia } from "elysia";
import { html } from "@elysiajs/html";
import { staticPlugin } from "@elysiajs/static";

const tools = [
  { category: "Filesystem", tools: ["eza", "bat", "fd", "duf", "dust", "trash_*"] },
  { category: "Search", tools: ["rg", "fzf_filter", "ast_grep"] },
  { category: "Git Forges", tools: ["gh_repo", "gh_issue", "gh_pr", "gh_search", "gh_release", "gh_workflow", "gh_run", "gh_api", "glab_issue", "glab_mr", "glab_pipeline"] },
  { category: "Containers", tools: ["podman", "dive", "skopeo", "crane", "trivy"] },
  { category: "Kubernetes", tools: ["kubectl_get", "kubectl_describe", "kubectl_logs", "kubectl_apply", "kubectl_delete", "kubectl_exec", "stern", "helm", "kustomize"] },
  { category: "Data Transform", tools: ["jq", "yq", "gron", "htmlq", "pup", "miller", "dasel"] },
  { category: "Network", tools: ["http", "dns", "usql"] },
  { category: "System", tools: ["procs", "tokei", "hyperfine"] },
  { category: "Diff/Git", tools: ["delta", "difft", "git_diff"] },
  { category: "Utilities", tools: ["tldr", "grex", "ouch_*", "pueue_*"] },
];

const totalTools = tools.reduce((acc, cat) => acc + cat.tools.length, 0);

const app = new Elysia()
  .use(html())
  .use(staticPlugin({ prefix: "/public" }))
  .get("/", () => page(homePage()))
  .get("/tools", () => page(toolsPage()))
  .get("/docs", () => page(docsPage()))
  .get("/api/tools", () => tools)
  .listen(3000);

console.log("Server running at http://localhost:" + app.server?.port);

function page(content: string) {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Modern CLI MCP</title>
  <link rel="stylesheet" href="/public/styles.css">
  <script src="https://unpkg.com/htmx.org@2.0.4"></script>
  <script src="https://unpkg.com/hyperscript.org@0.9.14"></script>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
  ${renderNav()}
  <main class="container mx-auto px-4 py-8">
    ${content}
  </main>
  ${renderFooter()}
</body>
</html>`;
}

function renderNav() {
  return `
  <nav class="bg-gray-800 border-b border-gray-700">
    <div class="container mx-auto px-4">
      <div class="flex items-center justify-between h-16">
        <a href="/" class="text-xl font-bold text-cyan-400">Modern CLI MCP</a>
        <div class="flex gap-6">
          <a href="/" class="hover:text-cyan-400 transition">Home</a>
          <a href="/tools" class="hover:text-cyan-400 transition">Tools</a>
          <a href="/docs" class="hover:text-cyan-400 transition">Docs</a>
          <a href="https://github.com/hellst0rm/modern-cli-mcp" class="hover:text-cyan-400 transition" target="_blank">GitHub</a>
        </div>
      </div>
    </div>
  </nav>`;
}

function renderFooter() {
  return `
  <footer class="bg-gray-800 border-t border-gray-700 mt-16 py-8">
    <div class="container mx-auto px-4 text-center text-gray-400">
      <p>MIT License</p>
    </div>
  </footer>`;
}

function homePage() {
  return `
  <div class="text-center py-16">
    <h1 class="text-5xl font-bold mb-4">Modern CLI MCP</h1>
    <p class="text-xl text-gray-400 mb-8">70+ CLI tools for AI/LLM agents via MCP</p>

    <div class="flex justify-center gap-4 mb-12">
      <a href="https://github.com/hellst0rm/modern-cli-mcp" class="bg-cyan-600 hover:bg-cyan-500 px-6 py-3 rounded-lg font-semibold transition">
        View on GitHub
      </a>
    </div>

    <div class="grid md:grid-cols-3 gap-6 max-w-4xl mx-auto">
      <div class="bg-gray-800 p-6 rounded-lg">
        <h3 class="text-xl font-semibold mb-2">${totalTools}+ Tools</h3>
        <p class="text-gray-400">Filesystem, Git, containers, Kubernetes, and more</p>
      </div>
      <div class="bg-gray-800 p-6 rounded-lg">
        <h3 class="text-xl font-semibold mb-2">JSON Output</h3>
        <p class="text-gray-400">Optimized for AI/LLM consumption</p>
      </div>
      <div class="bg-gray-800 p-6 rounded-lg">
        <h3 class="text-xl font-semibold mb-2">Zero Config</h3>
        <p class="text-gray-400">Nix bundles all dependencies</p>
      </div>
    </div>
  </div>

  <section class="mt-16">
    <h2 class="text-2xl font-bold mb-6 text-center">Quick Start</h2>
    <div class="bg-gray-800 p-6 rounded-lg max-w-2xl mx-auto">
      <p class="text-gray-400 mb-4">Add to your Claude Desktop config:</p>
      <pre class="bg-gray-900 p-4 rounded overflow-x-auto"><code class="text-cyan-400">{
  "mcpServers": {
    "modern-cli": {
      "command": "nix",
      "args": ["run", "github:hellst0rm/modern-cli-mcp", "--"]
    }
  }
}</code></pre>
    </div>
  </section>`;
}

function toolsPage() {
  const categoryCards = tools.map(cat => `
    <div class="bg-gray-800 p-6 rounded-lg">
      <h3 class="text-xl font-semibold mb-4 text-cyan-400">${cat.category}</h3>
      <div class="flex flex-wrap gap-2">
        ${cat.tools.map(t => `<span class="bg-gray-700 px-3 py-1 rounded text-sm">${t}</span>`).join("")}
      </div>
    </div>
  `).join("");

  return `
  <h1 class="text-3xl font-bold mb-8">Available Tools</h1>
  <p class="text-gray-400 mb-8">${totalTools}+ tools organized by category.</p>
  <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
    ${categoryCards}
  </div>`;
}

function docsPage() {
  return `
  <h1 class="text-3xl font-bold mb-8">Documentation</h1>

  <section class="mb-12">
    <h2 class="text-2xl font-semibold mb-4">Installation</h2>

    <h3 class="text-xl font-semibold mb-2 mt-6">Nix (Recommended)</h3>
    <pre class="bg-gray-800 p-4 rounded mb-4"><code>nix run github:hellst0rm/modern-cli-mcp</code></pre>

    <h3 class="text-xl font-semibold mb-2 mt-6">Docker</h3>
    <pre class="bg-gray-800 p-4 rounded mb-4"><code>docker run --rm -i ghcr.io/hellst0rm/modern-cli-mcp</code></pre>
  </section>

  <section class="mb-12">
    <h2 class="text-2xl font-semibold mb-4">Configuration</h2>
    <p class="text-gray-400 mb-4">Add to your Claude MCP configuration:</p>
    <pre class="bg-gray-800 p-4 rounded overflow-x-auto"><code>{
  "mcpServers": {
    "modern-cli": {
      "command": "nix",
      "args": ["run", "github:hellst0rm/modern-cli-mcp", "--"]
    }
  }
}</code></pre>
  </section>`;
}
