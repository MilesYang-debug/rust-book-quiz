// Schema validator for bank/chNN.json files. Run: node validate.js
"use strict";
const fs = require("fs");
const path = require("path");

const bankDir = path.join(__dirname, "bank");
const files = fs.readdirSync(bankDir).filter(f => /^ch\d+\.json$/.test(f)).sort();

const TAGS = new Set(["Concept", "Behavior", "Code Output", "Spot the Bug", "Misconception"]);
const errors = [];
const seenIds = new Set();
let totalQuestions = 0;

const bank = [];
for (const file of files) {
  try {
    bank.push(JSON.parse(fs.readFileSync(path.join(bankDir, file), "utf8")));
  } catch (e) {
    errors.push(`${file}: invalid JSON — ${e.message}`);
  }
}
console.log(`Loaded ${files.length} files, ${bank.length} chapter entries.`);

for (const ch of bank) {
  const label = `ch${String(ch.chapter).padStart(2, "0")}`;
  if (typeof ch.chapter !== "number") errors.push(`${label}: chapter must be a number`);
  if (!ch.title) errors.push(`${label}: missing title`);
  if (!Array.isArray(ch.questions) || ch.questions.length === 0) {
    errors.push(`${label}: no questions array`);
    continue;
  }
  totalQuestions += ch.questions.length;

  for (const q of ch.questions) {
    const where = `${label}/${q.id || "?"}`;
    if (!q.id) { errors.push(`${where}: missing id`); continue; }
    if (seenIds.has(q.id)) errors.push(`${where}: duplicate id`);
    seenIds.add(q.id);

    if (!q.prompt || typeof q.prompt !== "string") errors.push(`${where}: missing prompt`);
    if (!q.section) errors.push(`${where}: missing section`);
    if (!TAGS.has(q.tag)) errors.push(`${where}: bad tag "${q.tag}"`);
    if (![1, 2, 3].includes(q.difficulty)) errors.push(`${where}: bad difficulty ${q.difficulty}`);
    if (!q.explanation) errors.push(`${where}: missing explanation`);

    if (!q.options || typeof q.options !== "object") {
      errors.push(`${where}: missing options`);
      continue;
    }
    const keys = Object.keys(q.options);
    const validKeys = keys.every(k => /^[A-F]$/.test(k));
    if (!validKeys || keys.length < 3) errors.push(`${where}: bad option keys [${keys}]`);
    for (const k of keys) {
      if (typeof q.options[k] !== "string" || !q.options[k].trim()) errors.push(`${where}: empty option ${k}`);
    }

    if (Array.isArray(q.answer)) {
      if (q.answer.length < 2) errors.push(`${where}: multi answer has <2 entries`);
      for (const a of q.answer) if (!keys.includes(a)) errors.push(`${where}: answer ${a} not in options`);
      if (new Set(q.answer).size !== q.answer.length) errors.push(`${where}: duplicate answer letters`);
      if (keys.length < 5) errors.push(`${where}: multi-select should have 5 options, has ${keys.length}`);
    } else if (typeof q.answer === "string") {
      if (!keys.includes(q.answer)) errors.push(`${where}: answer "${q.answer}" not in options`);
    } else {
      errors.push(`${where}: answer must be string or array`);
    }

    if (q.code !== undefined && (typeof q.code !== "string" || !q.code.trim())) {
      errors.push(`${where}: code field present but empty/non-string`);
    }
  }
}

console.log(`Total questions: ${totalQuestions}`);
if (errors.length) {
  console.log(`\n${errors.length} problem(s):`);
  errors.forEach(e => console.log("  - " + e));
  process.exit(1);
} else {
  console.log("All checks passed.");
}
