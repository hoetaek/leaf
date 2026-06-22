#!/usr/bin/env node
// validate-manifests.mjs — keep the leaf plugin's dual-runtime manifests in sync.
//
// Checks (Node built-ins only, no deps):
//   1. Required fields present in each manifest.
//   2. plugin name/version/description agree across CC plugin.json, Codex
//      plugin.json, and both marketplace entries.
//   3. Both marketplaces register the `leaf` plugin and point at the same
//      ./plugins/leaf directory (single folder, dual manifest: CC + Codex
//      manifests live side by side; skills are not duplicated).
//   4. Every manifest carries the same plugin version (the plugin versions
//      independently of the leaf CLI; the canonical value is CC plugin.json's
//      `version`). The required leaf CLI floor is documented, not derived here.
//   5. plugins/leaf/skills/ contains skills with SKILL.md, including install
//      so Codex exposes the CLI installer through its supported skill surface.
//   6. Claude-style commands are still shipped for hosts that support them.
//      Codex loads only the skills its plugin.json declares and ignores the
//      shared dir's hooks/commands.
//
// Exits non-zero on any failure so CI blocks drift.

import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];
const fail = (m) => errors.push(m);

const readJSON = (rel) => {
  const p = join(ROOT, rel);
  if (!existsSync(p)) {
    fail(`missing file: ${rel}`);
    return null;
  }
  try {
    return JSON.parse(readFileSync(p, "utf8"));
  } catch (e) {
    fail(`invalid JSON in ${rel}: ${e.message}`);
    return null;
  }
};

const ccPlugin = readJSON("plugins/leaf/.claude-plugin/plugin.json");
const codexPlugin = readJSON("plugins/leaf/.codex-plugin/plugin.json");
const ccMarket = readJSON(".claude-plugin/marketplace.json");
const codexMarket = readJSON(".agents/plugins/marketplace.json");

// 1. Required fields.
if (ccPlugin && !ccPlugin.name) fail("CC plugin.json: missing `name`");
if (codexPlugin) {
  if (!codexPlugin.name) fail("Codex plugin.json: missing `name`");
  if (!codexPlugin.skills) fail("Codex plugin.json: missing `skills`");
  if (!codexPlugin.interface?.displayName)
    fail("Codex plugin.json: missing `interface.displayName`");
}
if (ccMarket) {
  if (!ccMarket.name) fail("CC marketplace.json: missing `name`");
  if (!ccMarket.owner?.name) fail("CC marketplace.json: missing `owner.name`");
  if (!Array.isArray(ccMarket.plugins) || ccMarket.plugins.length === 0)
    fail("CC marketplace.json: missing `plugins[]`");
}
if (codexMarket) {
  if (!codexMarket.name) fail("Codex marketplace.json: missing `name`");
  if (!Array.isArray(codexMarket.plugins) || codexMarket.plugins.length === 0)
    fail("Codex marketplace.json: missing `plugins[]`");
}

const ccEntry = ccMarket?.plugins?.find((p) => p.name === "leaf");
const codexEntry = codexMarket?.plugins?.find((p) => p.name === "leaf");
if (ccMarket && !ccEntry) fail("CC marketplace.json: no plugin entry named `leaf`");
if (codexMarket && !codexEntry) fail("Codex marketplace.json: no plugin entry named `leaf`");

// 2 + 4. Every manifest carries the same plugin version (independent of the CLI).
const pluginVersion = ccPlugin?.version;
const versions = {
  "CC plugin.json": ccPlugin?.version,
  "Codex plugin.json": codexPlugin?.version,
  "CC marketplace entry": ccEntry?.version,
  "Codex marketplace entry": codexEntry?.version,
  "CC marketplace metadata": ccMarket?.metadata?.version,
  "Codex marketplace metadata": codexMarket?.metadata?.version,
};
const missingVersions = Object.entries(versions)
  .filter(([, v]) => v == null)
  .map(([k]) => k);
if (missingVersions.length) fail(`missing version in: ${missingVersions.join(", ")}`);
const seen = new Set(Object.values(versions).filter(Boolean));
if (seen.size > 1) {
  fail(
    "version drift: " +
      Object.entries(versions)
        .map(([k, v]) => `${k}=${v ?? "?"}`)
        .join(", "),
  );
}

// name + description agreement.
const names = [ccPlugin?.name, codexPlugin?.name, ccEntry?.name, codexEntry?.name].filter(Boolean);
if (new Set(names).size > 1) fail(`plugin name mismatch: ${[...new Set(names)].join(", ")}`);

// 3. Marketplace sources point at the plugin dir.
if (ccEntry && ccEntry.source !== "./plugins/leaf")
  fail(`CC marketplace source should be "./plugins/leaf", got ${JSON.stringify(ccEntry.source)}`);
if (codexEntry && codexEntry.source?.path !== "./plugins/leaf")
  fail(
    `Codex marketplace source.path should be "./plugins/leaf", got ${JSON.stringify(codexEntry.source)}`,
  );

// 5. At least one skill with SKILL.md.
const skillsDir = join(ROOT, "plugins/leaf/skills");
if (!existsSync(skillsDir)) {
  fail("missing plugins/leaf/skills/");
} else {
  const skills = readdirSync(skillsDir).filter((d) => {
    const p = join(skillsDir, d);
    return statSync(p).isDirectory() && existsSync(join(p, "SKILL.md"));
  });
  let skillsOk = true;
  if (skills.length === 0) {
    fail("plugins/leaf/skills/ has no skill with a SKILL.md");
    skillsOk = false;
  }
  if (!skills.includes("install")) {
    fail("plugins/leaf/skills/install/SKILL.md is required for Codex install discovery");
    skillsOk = false;
  }
  if (!existsSync(join(skillsDir, "install", "agents", "openai.yaml"))) {
    fail("plugins/leaf/skills/install/agents/openai.yaml is required for Codex install display");
    skillsOk = false;
  } else {
    const installOpenaiYaml = readFileSync(
      join(skillsDir, "install", "agents", "openai.yaml"),
      "utf8",
    );
    if (!installOpenaiYaml.includes("allow_implicit_invocation: true")) {
      fail(
        "plugins/leaf/skills/install/agents/openai.yaml must set `allow_implicit_invocation: true` so Codex surfaces leaf:install with other LEAF skills",
      );
      skillsOk = false;
    }
  }
  const installDocs = [
    ["plugins/leaf/skills/install/SKILL.md", join(skillsDir, "install", "SKILL.md")],
    ["plugins/leaf/commands/install.md", join(ROOT, "plugins/leaf/commands/install.md")],
  ];
  for (const [rel, abs] of installDocs) {
    if (!existsSync(abs)) continue;
    const installDoc = readFileSync(abs, "utf8");
    if (installDoc.includes("cargo install --path")) {
      fail(`${rel}: install flow must not special-case source checkouts with cargo install --path`);
      skillsOk = false;
    }
    for (const required of ["macOS", "Linux", "Windows", "leaf --version"]) {
      if (!installDoc.includes(required)) {
        fail(`${rel}: install flow must mention ${required}`);
        skillsOk = false;
      }
    }
  }
  if (skillsOk) console.log(`✓ ${skills.length} skills: ${skills.join(", ")}`);
}

// Skills live once under plugins/leaf/skills/. Both runtimes install from the
// same ./plugins/leaf directory (single folder, dual manifest), so there is no
// second skills tree to mirror. Codex loads only what its plugin.json `skills`
// path declares and ignores hooks/commands in the shared dir.

const commandsDir = join(ROOT, "plugins/leaf/commands");
if (!existsSync(commandsDir)) {
  fail("missing plugins/leaf/commands/");
} else {
  const commands = readdirSync(commandsDir).filter((name) => name.endsWith(".md"));
  if (commands.length === 0) fail("plugins/leaf/commands/ has no markdown command files");
  else console.log(`✓ ${commands.length} commands: ${commands.join(", ")}`);
}

// --audit: scan every manifest JSON under the plugin/marketplace dirs for a
// `"version"` field and flag any that drifts from the plugin version. Catches a
// stray version in a NEW manifest the four hardcoded checks above don't know about.
if (process.argv.includes("--audit") && pluginVersion) {
  const SEMVER = /^\d+\.\d+\.\d+/;
  const collectJson = (rel) => {
    const abs = join(ROOT, rel);
    if (!existsSync(abs)) return [];
    if (statSync(abs).isDirectory()) {
      return readdirSync(abs).flatMap((name) => collectJson(join(rel, name)));
    }
    return rel.endsWith(".json") ? [rel] : [];
  };
  const findVersions = (node, path, out) => {
    if (Array.isArray(node)) {
      node.forEach((v, i) => findVersions(v, `${path}[${i}]`, out));
    } else if (node && typeof node === "object") {
      for (const [k, v] of Object.entries(node)) {
        if (k === "version" && typeof v === "string" && SEMVER.test(v)) {
          out.push({ path: `${path}.version`, value: v });
        } else {
          findVersions(v, path ? `${path}.${k}` : k, out);
        }
      }
    }
  };
  const manifestFiles = [
    ".claude-plugin",
    ".agents/plugins",
    "plugins/leaf/.claude-plugin",
    "plugins/leaf/.codex-plugin",
  ].flatMap(collectJson);
  let audited = 0;
  for (const rel of manifestFiles) {
    let json;
    try {
      json = JSON.parse(readFileSync(join(ROOT, rel), "utf8"));
    } catch {
      continue;
    }
    const found = [];
    findVersions(json, "", found);
    for (const { path, value } of found) {
      audited += 1;
      if (value !== pluginVersion)
        fail(`audit: ${rel} ${path}=${value} != plugin ${pluginVersion}`);
    }
  }
  console.log(`✓ audit: ${audited} version field(s) across manifests match ${pluginVersion}`);
}

if (errors.length) {
  console.error("✗ manifest validation failed:");
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}
console.log(`✓ manifests consistent at plugin version ${pluginVersion}`);
