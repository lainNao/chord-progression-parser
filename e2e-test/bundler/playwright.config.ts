import type { PlaywrightTestConfig } from "@playwright/test";
import { devices } from "@playwright/test";

// https://playwright.dev/docs/test-configuration
export default <PlaywrightTestConfig>{
  // テスト開始の一番最初に発火させる処理を書いたファイル
  // 認証情報をファイルに保存して、皇族のテストで使い回すために使ったりする
  // globalSetup: require.resolve("./src/e2e-tests/global-setup.ts"),

  // テストを置くフォルダ
  testDir: "./src",
  // スクリーンショットやトレース結果等を入れるフォルダ
  outputDir: "./e2e-test-results",
  fullyParallel: true,
  //1テストのタイムアウト時間。5秒
  timeout: 1000 * 5,
  //全テストの総タイムアウト時間。1分
  globalTimeout: 1000 * 1 * 60,
  expect: {
    //expect()のタイムアウト時間。5秒
    timeout: 1000 * 5,
  },

  // テスト実行時に起動するサーバーの設定
  webServer: {
    command: "bun dev",
    port: 3034,
    reuseExistingServer: true,
  },

  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: true,
  retries: 0,
  reporter: "html",

  // projectsの共通設定 https://playwright.dev/docs/api/class-testoptions
  use: {
    actionTimeout: 0, //click()等のアクションのタイムアウト時間。0は無制限

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: "on-first-retry",
  },

  // 使うブラウザの設定。増やすと並列実行される
  projects: [
    // -------------------デスクトップ------------------- //
    {
      name: "Google Chrome",
      use: {
        channel: "chrome",
      },
    },
    {
      name: "Microsoft Edge",
      use: {
        channel: "msedge",
      },
    },
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
      },
    },

    {
      name: "webkit",
      use: {
        ...devices["Desktop Safari"],
      },
    },

    // -------------------スマホ------------------- //
    {
      name: "Mobile Chrome",
      use: {
        ...devices["Pixel 5"],
      },
    },
    {
      name: "Mobile Safari",
      use: {
        ...devices["iPhone 12"],
      },
    },
  ],
};
