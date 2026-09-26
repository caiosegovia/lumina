// Real packaged application smoke. Supply extracted MSI executable as argv[2].
// Everything is generated under artifacts; no existing catalog is opened.
import { chromium } from "@playwright/test";
import { spawn, execFileSync } from "node:child_process";
import {
  mkdirSync,
  copyFileSync,
  readFileSync,
  existsSync,
  writeFileSync,
} from "node:fs";
import { resolve, join, dirname } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import assert from "node:assert/strict";
const exe = resolve(process.argv[2] || "src-tauri/target/release/lumina.exe");
const root = resolve(`artifacts/0.26/desktop-${Date.now()}`),
  source = join(root, "source"),
  master = join(root, "master"),
  backup = join(root, "replica"),
  profile = join(root, "profile");
for (const dir of [source, master, backup, profile])
  mkdirSync(dir, { recursive: true });
const ffmpeg = resolve("src-tauri/tools/ffmpeg.exe");
for (const color of ["red", "green", "blue"])
  execFileSync(
    ffmpeg,
    [
      "-v",
      "error",
      "-y",
      "-f",
      "lavfi",
      "-i",
      `color=c=${color}:s=1600x1200`,
      "-frames:v",
      "1",
      join(source, `${color}.jpg`),
    ],
    { windowsHide: true },
  );
copyFileSync(join(source, "red.jpg"), join(source, "red-copy.jpg"));
execFileSync(
  ffmpeg,
  [
    "-v",
    "error",
    "-y",
    "-f",
    "lavfi",
    "-i",
    "testsrc=size=640x360:rate=24",
    "-t",
    "3",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    join(source, "video.mp4"),
  ],
  { windowsHide: true },
);
const original = readFileSync(join(source, "red.jpg"));
const app = spawn(exe, [], {
  cwd: dirname(exe),
  windowsHide: true,
  env: {
    ...process.env,
    LUMINA_DATA_DIR: profile,
    WEBVIEW2_USER_DATA_FOLDER: join(root, "webview"),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: "--remote-debugging-port=9237",
  },
});
let browser,
  finished = false;
try {
  for (let attempt = 0; attempt < 120; attempt++) {
    try {
      browser = await chromium.connectOverCDP("http://127.0.0.1:9237", {
        timeout: 1000,
      });
      break;
    } catch {
      if (app.exitCode !== null) throw Error(`App exited: ${app.exitCode}`);
      await delay(500);
    }
  }
  assert(browser, "WebView2 debugging endpoint did not become ready");
  const context = browser.contexts()[0];
  let page = context.pages()[0];
  if (!page) page = await context.waitForEvent("page");
  const errors = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  await page.getByLabel("Pasta-mestre").fill(master);
  await page.getByLabel("Pasta de backup").fill(backup);
  await page.getByRole("button", { name: /Criar biblioteca/ }).click();
  await page.getByRole("button", { name: "Biblioteca", exact: true }).waitFor();
  assert(
    existsSync(join(profile, "Lumina", "library.json")),
    "Isolated config was not created",
  );
  const invoke = (command, args = {}) =>
    page.evaluate(
      ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
      { command, args },
    );
  const config = JSON.parse(
    readFileSync(join(profile, "Lumina", "library.json"), "utf8"),
  );
  assert.equal(config.masterPath.replace(/^\\\\\?\\/, ""), master);
  const jobId = await invoke("start_analysis", {
    sourcePath: source,
    sourceName: "Synthetic smoke 0.26",
  });
  const until = async (check, label, timeout = 180000) => {
    const end = Date.now() + timeout;
    while (Date.now() < end) {
      const result = await check();
      if (result) return result;
      await delay(500);
    }
    throw Error(`Timeout: ${label}`);
  };
  await until(async () => {
    const p = await invoke("get_job_progress", { jobId });
    if (["failed", "interrupted"].includes(p.state))
      throw Error(JSON.stringify(p));
    return p.state === "ready";
  }, "analysis");
  const summary = await invoke("get_import_summary", { jobId });
  assert.equal(summary.discovered, 5);
  assert.equal(summary.invalid, 0);
  console.log("PASS real analysis of synthetic photo/video/duplicate");
  await invoke("start_consolidation", { jobId });
  await until(async () => {
    const p = await invoke("get_job_progress", { jobId });
    if (["failed", "interrupted", "backup_error"].includes(p.state))
      throw Error(JSON.stringify(p));
    return p.state === "completed" || p.state === "protection_pending";
  }, "consolidation");
  const gallery = () =>
    invoke("search_gallery", {
      request: { filters: { query: "" }, limit: 100 },
    });
  let assets = (await gallery()).assets;
  assert.equal(assets.length, 4);
  if (assets.some((a) => a.protectionState !== "replica_verified")) {
    await delay(1000);
    await invoke("start_protection", { jobId });
  }
  await until(async () => {
    assets = (await gallery()).assets;
    return assets.every((a) => a.protectionState === "replica_verified");
  }, "protection");
  console.log(
    "PASS real consolidation, exact deduplication and verified replica",
  );
  const indexed = await invoke("build_discovery_index");
  assert.equal(indexed.indexed, 3);
  assert.equal(indexed.failed, 0);
  assert.equal((await invoke("build_discovery_index")).indexed, 0);
  const failures = await invoke("get_technical_failures", { offset: 0 });
  assert.equal(failures.total, 0);
  const photo = assets.find((a) => a.mediaType === "photo");
  await page.getByRole("button", { name: "Biblioteca", exact: true }).click();
  await page
    .getByRole("button", {
      name: `Abrir detalhes de ${photo.filename}`,
      exact: true,
    })
    .click();
  await page.waitForFunction(
    () => document.querySelector(".media-viewport img")?.naturalWidth > 0,
  );
  await page
    .getByRole("button", { name: "Aumentar zoom", exact: true })
    .click();
  await page.waitForFunction(
    () =>
      document.querySelector('[aria-label="Nível de zoom"]')?.textContent ===
      "125%",
  );
  await page
    .getByRole("button", { name: "Abrir em tela cheia", exact: true })
    .click();
  await page.screenshot({ path: join(root, "packaged-viewer.png") });
  await page.getByRole("button",{name:"Sair da tela cheia",exact:true}).click();
  await page.locator(".inspector-heading .close").click();
  const videoAsset=assets.find(asset=>asset.mediaType==="video");
  await page.getByRole("button",{name:`Abrir detalhes de ${videoAsset.filename}`,exact:true}).click();
  await page.waitForFunction(()=>document.querySelector(".media-viewport video")?.videoWidth>0);
  await page.getByRole("button",{name:"Reproduzir vídeo",exact:true}).click();
  await page.waitForFunction(()=>document.querySelector(".media-viewport video")?.currentTime>0);
  await page.getByRole("button",{name:"Aumentar zoom",exact:true}).click();
  await page.waitForFunction(()=>document.querySelector(".media-viewport video")?.style.transform.includes("scale(1.25)"));
  await page.getByRole("button",{name:"Abrir em tela cheia",exact:true}).click();
  await page.screenshot({path:join(root,"packaged-video.png")});
  assert.deepEqual(readFileSync(join(source, "red.jpg")), original);
  assert.deepEqual(errors, []);
  // Request the real native close event, not process termination.
  execFileSync(
    "powershell.exe",
    [
      "-NoProfile",
      "-Command",
      `$p=Get-Process -Id ${app.pid}; $null=$p.CloseMainWindow(); if(-not $p.WaitForExit(15000)){exit 1}`,
    ],
    { windowsHide: true },
  );
  assert(
    !existsSync(join(profile, "Lumina", "diagnostics", "running.session")),
    "Session marker survived graceful exit",
  );
  writeFileSync(
    join(root, "result.json"),
    JSON.stringify(
      {
        version: "0.26.0-beta.1",
        executable: exe,
        isolatedProfile: true,
        sourceFiles: 5,
        assets: assets.length,
        indexed: indexed.indexed,
        technicalFailures: failures.total,
        originalUnchanged: true,
        gracefulExit: true,
        videoPlaybackAndZoom: true,
      },
      null,
      2,
    ),
  );
  finished = true;
  console.log(
    `PASS packaged viewer, original unchanged, clean shutdown. Evidence: ${root}`,
  );
} finally {
  // Never terminate an unrelated Lumina instance. Only this spawned test child.
  if (!finished && app.exitCode === null) app.kill();
  if (browser) await browser.close().catch(() => {});
}
