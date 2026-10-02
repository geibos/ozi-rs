// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import type { FtpAccountDto } from "../lib/bindings";

/**
 * The FTP section of the settings screen sends what the operator typed, and a
 * password only when one was typed: an empty password field on an existing
 * account means "keep the stored one", not "clear it".
 */
const api = vi.hoisted(() => ({
  accounts: [] as FtpAccountDto[],
  saveFtpAccount: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("$lib/api", () => ({
  listFtpAccounts: async () => api.accounts,
  saveFtpAccount: (...args: unknown[]) => api.saveFtpAccount(...args),
  deleteFtpAccount: vi.fn(async () => {}),
  checkFtpAccount: vi.fn(async () => ({
    outcome: "ok",
    detail: null,
    folder: "/",
  })),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  confirm: vi.fn(async () => true),
}));

vi.mock("svelte-sonner", () => ({
  toast: Object.assign(vi.fn(), {
    success: vi.fn(),
    error: (...args: unknown[]) => api.toastError(...args),
  }),
}));

import FtpAccounts from "../components/FtpAccounts.svelte";
import { setLocale } from "../lib/i18n";

async function settle() {
  for (let i = 0; i < 5; i += 1) await tick();
}

function field(label: string): HTMLInputElement {
  return screen.getByLabelText(label) as HTMLInputElement;
}

describe("the FTP accounts on the settings screen", () => {
  beforeEach(() => {
    setLocale("ru");
    api.accounts = [];
    api.saveFtpAccount.mockReset().mockImplementation(async (input) => ({
      ...input,
      id: input.id ?? "new",
      has_password: true,
    }));
    api.toastError.mockReset();
  });

  afterEach(() => cleanup());

  it("sends a new result endpoint as typed, with its password", async () => {
    render(FtpAccounts);
    await settle();
    await fireEvent.click(screen.getByText("Добавить точку выгрузки"));
    await settle();

    expect(field("Название").value).toBe("Контур 1");
    await fireEvent.input(field("Сервер"), {
      target: { value: "results.example.org" },
    });
    await fireEvent.input(field("Логин"), { target: { value: "crew1" } });
    await fireEvent.input(field("Пароль"), { target: { value: "секрет" } });
    await fireEvent.input(field("Папка"), { target: { value: "/incoming" } });
    await fireEvent.click(screen.getByText("Сохранить"));
    await settle();

    expect(api.saveFtpAccount).toHaveBeenCalledWith(
      {
        id: null,
        role: "results",
        name: "Контур 1",
        host: "results.example.org",
        port: 21,
        login: "crew1",
        folder: "/incoming",
      },
      "секрет",
    );
  });

  it("keeps the stored password when the field is left empty", async () => {
    api.accounts = [
      {
        id: "b1",
        role: "bundles",
        name: "Комплекты",
        host: "maps.example.org",
        port: 21,
        login: "maps",
        folder: "/",
        has_password: true,
      },
    ];
    render(FtpAccounts);
    await settle();
    await fireEvent.click(screen.getByText("Изменить"));
    await settle();

    expect(field("Пароль").placeholder).toBe(
      "Сохранён — оставьте пустым, чтобы не менять",
    );
    await fireEvent.input(field("Сервер"), {
      target: { value: "maps2.example.org" },
    });
    await fireEvent.click(screen.getByText("Сохранить"));
    await settle();

    expect(api.saveFtpAccount).toHaveBeenCalledWith(
      expect.objectContaining({ id: "b1", host: "maps2.example.org" }),
      null,
    );
  });

  it("says which field is wrong, in the operator's language", async () => {
    api.saveFtpAccount.mockRejectedValue("ftp.error.hostNotAName");
    render(FtpAccounts);
    await settle();
    await fireEvent.click(screen.getByText("Добавить"));
    await settle();
    await fireEvent.input(field("Сервер"), {
      target: { value: "maps.example.org/bundles" },
    });
    await fireEvent.click(screen.getByText("Сохранить"));
    await settle();

    expect(api.toastError).toHaveBeenCalledWith(
      "Не удалось сохранить учётную запись FTP",
      {
        description: "Только имя сервера — для порта и папки есть свои поля",
      },
    );
  });

  it("offers one bundle account and any number of result endpoints", async () => {
    api.accounts = [
      {
        id: "b1",
        role: "bundles",
        name: "Комплекты",
        host: "maps.example.org",
        port: 21,
        login: "maps",
        folder: "/",
        has_password: true,
      },
      ...[1, 2, 3, 4].map((n) => ({
        id: `r${n}`,
        role: "results" as const,
        name: `Контур ${n}`,
        host: `c${n}.example.org`,
        port: 2121,
        login: `crew${n}`,
        folder: "/",
        has_password: n !== 4,
      })),
    ];
    render(FtpAccounts);
    await settle();

    expect(screen.getAllByTestId("ftp-account")).toHaveLength(5);
    // The bundle account is there, so it is not offered again.
    expect(screen.queryByText("Добавить")).toBeNull();
    expect(screen.getByText("Добавить точку выгрузки")).toBeTruthy();
    expect(screen.getByText("crew4@c4.example.org:2121/")).toBeTruthy();
    expect(screen.getAllByText("без пароля")).toHaveLength(1);
  });
});
