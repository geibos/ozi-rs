// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render, cleanup } from "@testing-library/svelte";
import { afterEach } from "vitest";

import * as Button from "$lib/components/ui/button";
import * as Card from "$lib/components/ui/card";
import * as Dialog from "$lib/components/ui/dialog";
import * as Input from "$lib/components/ui/input";
import * as Label from "$lib/components/ui/label";
import * as Popover from "$lib/components/ui/popover";
import * as ScrollArea from "$lib/components/ui/scroll-area";
import * as Select from "$lib/components/ui/select";
import * as Separator from "$lib/components/ui/separator";
import * as Slider from "$lib/components/ui/slider";
import * as Sonner from "$lib/components/ui/sonner";
import * as Switch from "$lib/components/ui/switch";
import * as Table from "$lib/components/ui/table";
import * as Tabs from "$lib/components/ui/tabs";
import * as Tooltip from "$lib/components/ui/tooltip";
import { cn } from "$lib/utils";

afterEach(cleanup);

describe("shadcn-svelte primitives", () => {
  it("re-export `cn` and the helper composes Tailwind classes", () => {
    expect(cn("p-2", "p-4")).toContain("p-4");
    expect(cn("text-foreground", undefined, "bg-background")).toMatch(
      /text-foreground/,
    );
  });

  const cases: Array<[string, unknown]> = [
    ["button", Button.Button],
    ["card", Card.Root ?? Card.Card],
    ["card-content", Card.Content],
    ["dialog", Dialog.Root],
    ["dialog-content", Dialog.Content],
    ["input", Input.Root],
    ["label", Label.Root],
    ["popover", Popover.Root],
    ["popover-content", Popover.Content],
    ["scroll-area", ScrollArea.Root],
    ["select", Select.Root],
    ["select-trigger", Select.Trigger],
    ["separator", Separator.Root],
    ["slider", Slider.Root],
    ["sonner", Sonner.Toaster],
    ["switch", Switch.Root],
    ["table", Table.Root],
    ["table-header", Table.Header],
    ["table-row", Table.Row],
    ["tabs", Tabs.Root],
    ["tabs-list", Tabs.List],
    ["tooltip", Tooltip.Root],
  ];

  for (const [name, exportRef] of cases) {
    it(`exports a component constructor for ${name}`, () => {
      expect(exportRef, name).toBeDefined();
      expect(typeof exportRef).toBe("function");
    });
  }

  it("renders <Button> without throwing", () => {
    const { container } = render(Button.Button, { props: {} });
    expect(container.querySelector("button")).not.toBeNull();
  });

  it("renders <Input> without throwing", () => {
    const { container } = render(Input.Root, { props: {} });
    expect(container.querySelector("input")).not.toBeNull();
  });

  it("renders <Label> without throwing", () => {
    const { container } = render(Label.Root, { props: {} });
    expect(container.querySelector("label")).not.toBeNull();
  });

  it("renders <Separator> without throwing", () => {
    const { container } = render(Separator.Root, { props: {} });
    expect(container.firstElementChild).not.toBeNull();
  });
});
