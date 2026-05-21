import { chromium, webkit } from "playwright";
import { execSync } from "node:child_process";
import { existsSync } from "node:fs";

const URL = "http://127.0.0.1:8080";
const TIMEOUT_MS = 180_000;

async function verifyPlaywright(name, browserType, launchOptions = {}) {
  const consoleErrors = [];
  const pageErrors = [];
  let browser;

  try {
    browser = await browserType.launch({
      headless: true,
      args: ["--enable-unsafe-webgpu", "--use-angle=metal"],
      ...launchOptions,
    });
    const page = await browser.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text());
    });
    page.on("pageerror", (err) => pageErrors.push(String(err)));

    await page.goto(URL, { waitUntil: "domcontentloaded", timeout: TIMEOUT_MS });

    await page.waitForFunction(
      () => {
        const loading = document.getElementById("gravitium-loading");
        const noWebgpu = document.getElementById("gravitium-no-webgpu");
        return (
          loading?.hasAttribute("hidden") && noWebgpu?.hasAttribute("hidden")
        );
      },
      { timeout: TIMEOUT_MS },
    );

    await page.waitForTimeout(3000);

    const state = await page.evaluate(() => {
      const canvas = document.getElementById("gravitium-canvas");
      return {
        webgpu: !!navigator.gpu,
        canvasWidth: canvas?.width ?? 0,
        canvasHeight: canvas?.height ?? 0,
        title: document.title,
      };
    });

    const screenshotPath = `/tmp/gravitium-${name}.png`;
    await page.screenshot({ path: screenshotPath, fullPage: false });

    const fatal = [...consoleErrors, ...pageErrors].filter(
      (e) =>
        /panicked|wgpu|WebGPU|surface|fatal/i.test(e) &&
        !/favicon|404/i.test(e),
    );

    const ok =
      state.webgpu &&
      state.canvasWidth > 0 &&
      state.canvasHeight > 0 &&
      fatal.length === 0;

    return {
      name,
      ok,
      state,
      fatal,
      screenshotPath,
    };
  } catch (err) {
    return {
      name,
      ok: false,
      error: String(err),
      fatal: [...consoleErrors, ...pageErrors],
    };
  } finally {
    await browser?.close();
  }
}

function verifySafariApp() {
  try {
    execSync("open -a Safari " + URL, { stdio: "ignore" });
  } catch (err) {
    return { name: "safari-app", ok: false, error: String(err) };
  }

  execSync("sleep 20");

  let url = "";
  try {
    url = execSync(
      'osascript -e \'tell application "Safari" to get URL of current tab of front window\'',
      { encoding: "utf8" },
    ).trim();
  } catch (err) {
    return { name: "safari-app", ok: false, error: `AppleScript: ${err}` };
  }

  execSync("screencapture -x /tmp/gravitium-safari-app.png");

  try {
    execSync('osascript -e \'tell application "Safari" to close front window\'', {
      stdio: "ignore",
    });
  } catch {
    // ignore
  }

  const ok = url.includes("127.0.0.1:8080") || url.includes("localhost:8080");
  return {
    name: "safari-app",
    ok,
    state: { url },
    screenshotPath: "/tmp/gravitium-safari-app.png",
  };
}

const results = [];

results.push(
  await verifyPlaywright("chrome", chromium, { channel: "chrome" }),
);
results.push(await verifyPlaywright("webkit", webkit));
results.push(verifySafariApp());

let failed = false;
for (const r of results) {
  console.log(JSON.stringify(r, null, 2));
  if (!r.ok) failed = true;
}

if (failed) process.exit(1);
