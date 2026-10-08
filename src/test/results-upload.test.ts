// @vitest-environment jsdom
/**
 * «Отправить на сервер»: the dialog shows what goes where, names a file with
 * the finding, and makes the search's folder only after asking (standard
 * п. 33–36).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { tick } from "svelte";

const { getResultsUploadPlan, listFtpAccounts, uploadResultsFtp } = vi.hoisted(
  () => ({
    getResultsUploadPlan: vi.fn(),
    listFtpAccounts: vi.fn(),
    uploadResultsFtp: vi.fn(),
  }),
);

const { openDialog } = vi.hoisted(() => ({ openDialog: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openDialog }));

vi.mock("$lib/api", () => ({
  getResultsUploadPlan,
  listFtpAccounts,
  uploadResultsFtp,
}));

import ResultsUpload from "../components/ResultsUpload.svelte";
import { resultsUploadOpen } from "../lib/stores";
import { setLocale } from "../lib/i18n";

const account = {
  id: "c1",
  role: "results",
  name: "Контур 1",
  host: "ftp.example.org",
  port: 21,
  login: "crew",
  folder: "/results",
  has_password: true,
};

describe("sending a search's results", () => {
  beforeEach(() => {
    setLocale("ru");
    vi.clearAllMocks();
    listFtpAccounts.mockResolvedValue([
      account,
      { ...account, id: "b", role: "bundles", name: "Комплекты" },
    ]);
    getResultsUploadPlan.mockResolvedValue({
      search_folder: "2026-10-08_Mesto",
      dir: "/b/2026-10-08_Mesto/10-Tracks",
      files: ["20261008_Lisa15.plt", "Waypoints_20261008.wpt"],
      with_bvp: ["Waypoints_20261008.wpt"],
    });
  });

  afterEach(() => {
    resultsUploadOpen.set(false);
    cleanup();
  });

  async function open() {
    render(ResultsUpload);
    resultsUploadOpen.set(true);
    await vi.waitFor(() => screen.getByTestId("results-upload-files"));
  }

  it("shows the folder, the files and the finding before anything is sent", async () => {
    await open();
    expect(screen.getByTestId("results-upload-remote").textContent).toContain(
      "/results/2026-10-08_Mesto",
    );
    expect(screen.getByTestId("results-upload-files").textContent).toContain(
      "20261008_Lisa15.plt",
    );
    expect(screen.getByTestId("results-upload-bvp").textContent).toContain(
      "п. 36",
    );
    // Only results accounts are offered.
    expect(
      screen.getByTestId("results-upload-account").querySelectorAll("option"),
    ).toHaveLength(1);
    expect(uploadResultsFtp).not.toHaveBeenCalled();
  });

  it("asks before making the search's folder, then sends", async () => {
    uploadResultsFtp
      .mockResolvedValueOnce({
        outcome: "no_search_folder",
        remote: "/results/2026-10-08_Mesto",
        uploaded: [],
        detail: null,
      })
      .mockResolvedValueOnce({
        outcome: "done",
        remote: "/results/2026-10-08_Mesto",
        uploaded: ["20261008_Lisa15.plt", "Waypoints_20261008.wpt"],
        detail: null,
      });
    await open();

    await fireEvent.click(screen.getByTestId("results-upload-send"));
    await vi.waitFor(() => screen.getByTestId("results-upload-ask-folder"));
    expect(uploadResultsFtp).toHaveBeenCalledWith("c1", false, null);

    await fireEvent.click(screen.getByTestId("results-upload-create"));
    await vi.waitFor(() =>
      expect(uploadResultsFtp).toHaveBeenLastCalledWith("c1", true, null),
    );
    await tick();
    expect(get(resultsUploadOpen)).toBe(false);
  });

  it("says what the server said when it refuses", async () => {
    uploadResultsFtp.mockResolvedValueOnce({
      outcome: "login_refused",
      remote: null,
      uploaded: [],
      detail: "530 Authentication failed",
    });
    await open();
    await fireEvent.click(screen.getByTestId("results-upload-send"));
    await vi.waitFor(() =>
      expect(
        screen.getByTestId("results-upload-problem").textContent,
      ).toContain("530 Authentication failed"),
    );
  });

  it("with no bundle open, sends from a search folder the operator picks", async () => {
    getResultsUploadPlan.mockResolvedValueOnce(null).mockResolvedValueOnce({
      search_folder: "2026-10-08_Mesto",
      dir: "/searches/2026-10-08_Mesto/10-Tracks",
      files: ["20261008_Lisa15.plt"],
      with_bvp: [],
    });
    openDialog.mockResolvedValueOnce("/searches/2026-10-08_Mesto");
    uploadResultsFtp.mockResolvedValueOnce({
      outcome: "done",
      remote: "/results/2026-10-08_Mesto",
      uploaded: ["20261008_Lisa15.plt"],
      detail: null,
    });
    render(ResultsUpload);
    resultsUploadOpen.set(true);
    await vi.waitFor(() => screen.getByTestId("results-upload-pick"));
    await fireEvent.click(screen.getByTestId("results-upload-pick"));
    await vi.waitFor(() => screen.getByTestId("results-upload-files"));
    expect(getResultsUploadPlan).toHaveBeenLastCalledWith(
      "/searches/2026-10-08_Mesto",
    );
    await fireEvent.click(screen.getByTestId("results-upload-send"));
    await vi.waitFor(() =>
      expect(uploadResultsFtp).toHaveBeenCalledWith(
        "c1",
        false,
        "/searches/2026-10-08_Mesto",
      ),
    );
  });
});
