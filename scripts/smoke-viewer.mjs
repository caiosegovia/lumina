// Requires npm run dev on port 1420. Uses synthetic media, never a real catalog.
import { chromium } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import assert from "node:assert/strict";
mkdirSync("artifacts/0.26", { recursive: true });
execFileSync(
  "src-tauri/tools/ffmpeg.exe",
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
    "libvpx",
    "-pix_fmt",
    "yuv420p",
    "artifacts/0.26/smoke.webm",
  ],
  { windowsHide: true },
);
const browser = await chromium.launch({ channel: "msedge", headless: true });
try {
  for (const [width, height, scale] of [
    [1400, 900, 1],
    [1100, 700, 1.25],
    [1000, 700, 1.5],
  ]) {
    const page = await browser.newPage({
      viewport: { width, height },
      deviceScaleFactor: scale,
    });
    const errors = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    await page.goto("http://localhost:1420");
    if (scale > 1)
      await page.evaluate(() => {
        document.documentElement.dataset.theme = "dark";
      });
    await page.evaluate(async () => {
      const { api } = await import("/src/api.ts");
      const canvas = document.createElement("canvas");
      canvas.width = 4000;
      canvas.height = 3000;
      const ctx = canvas.getContext("2d");
      ctx.fillStyle = "#438871";
      ctx.fillRect(0, 0, 4000, 3000);
      ctx.fillStyle = "#eed9ba";
      ctx.fillRect(1000, 500, 1500, 2000);
      const photo = canvas.toDataURL("image/jpeg");
      api.photoPreview = async () => photo;
      api.thumbnail = async () => photo;
      api.mediaUrl = async () => "/artifacts/0.26/smoke.webm";
    });
    await page.getByLabel("Pasta-mestre").fill("D:\\Teste\\Master");
    await page.getByLabel("Pasta de backup").fill("E:\\Teste\\Replica");
    await page.getByRole("button", { name: /Criar biblioteca/ }).click();
    await page.getByRole("button", { name: "Biblioteca", exact: true }).click();
    await page
      .getByRole("button", {
        name: "Abrir detalhes de IMG_2401.JPG",
        exact: true,
      })
      .click();
    await page.locator(".media-viewport img").waitFor();
    await page.waitForFunction(
      () =>
        document.querySelector(".media-viewport img")?.naturalWidth === 4000,
    );
    const fit = await page.locator(".media-viewport img").boundingBox();
    await page
      .getByRole("button", { name: "Aumentar zoom", exact: true })
      .click();
    await page.waitForFunction(
      () =>
        document.querySelector('[aria-label="Nível de zoom"]')?.textContent ===
        "125%",
    );
    const zoomed = await page.locator(".media-viewport img").boundingBox();
    assert(zoomed.width > fit.width * 1.2);
    await page
      .getByRole("button", { name: "Abrir em tela cheia", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Preencher área", exact: true })
      .click();
    await page.locator(".media-viewport").hover();
    await page.mouse.wheel(0, -120);
    const checkControls = async () => {
      for (const name of ["Aumentar zoom", "Diminuir zoom", "Próxima mídia"]) {
        const r = await page
          .getByRole("button", { name, exact: true })
          .boundingBox();
        assert(
          r &&
            r.x >= 0 &&
            r.y >= 0 &&
            r.x + r.width <= width + 1 &&
            r.y + r.height <= height + 1,
          `${name} outside ${width}x${height}: ${JSON.stringify(r)}`,
        );
      }
    };
    await checkControls();
    await page.screenshot({ path: `artifacts/0.26/photo-${width}.png` });
    await page
      .getByRole("button", { name: "Sair da tela cheia", exact: true })
      .click();
    await page
      .getByRole("button", { name: "Mídia anterior", exact: true })
      .click();
    await page.locator(".media-viewport video").waitFor();
    await page.waitForFunction(
      () => document.querySelector(".media-viewport video")?.videoWidth > 0,
    );
    await page
      .getByRole("button", { name: "Reproduzir vídeo", exact: true })
      .click();
    await page.waitForFunction(
      () => document.querySelector(".media-viewport video")?.currentTime > 0,
    );
    await page
      .getByRole("button", { name: "Aumentar zoom", exact: true })
      .click();
    await page.waitForFunction(() =>
      document
        .querySelector(".media-viewport video")
        ?.style.transform.includes("scale(1.25)"),
    );
    await checkControls();
    await page
      .getByRole("button", { name: "Abrir em tela cheia", exact: true })
      .click();
    await checkControls();
    await page.screenshot({ path: `artifacts/0.26/video-${width}.png` });
    assert.deepEqual(errors, []);
    await page.close();
    console.log(
      `PASS photo/video zoom, fullscreen, controls and playback ${width}x${height}`,
    );
  }
} finally {
  await browser.close();
}
