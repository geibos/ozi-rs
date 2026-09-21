// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { isEditableTarget } from "../lib/editable-target";

/**
 * A global chord must not hijack native text editing. Backspace while renaming
 * a track deletes a character, not the last measured point; Cmd+Z there is the
 * input's own undo.
 */
function element(html: string): HTMLElement {
  const host = document.createElement("div");
  host.innerHTML = html;
  return host.firstElementChild as HTMLElement;
}

describe("what counts as something the operator is typing into", () => {
  it("is an input, a textarea or a select", () => {
    expect(isEditableTarget(element("<input />"))).toBe(true);
    expect(isEditableTarget(element("<textarea></textarea>"))).toBe(true);
    expect(isEditableTarget(element("<select></select>"))).toBe(true);
  });

  it("is a contenteditable, written either way", () => {
    expect(
      isEditableTarget(element('<div contenteditable="true"></div>')),
    ).toBe(true);
    expect(isEditableTarget(element('<div contenteditable=""></div>'))).toBe(
      true,
    );
  });

  /** The event's target is often a child of the field, not the field itself. */
  it("looks up the tree, not only at the element itself", () => {
    const wrapper = element('<div contenteditable="true"><span></span></div>');
    expect(isEditableTarget(wrapper.querySelector("span"))).toBe(true);
  });

  it("is not a button, a map canvas or nothing at all", () => {
    expect(isEditableTarget(element("<button></button>"))).toBe(false);
    expect(isEditableTarget(element("<canvas></canvas>"))).toBe(false);
    expect(isEditableTarget(null)).toBe(false);
  });

  it("is not a contenteditable that is explicitly off", () => {
    expect(
      isEditableTarget(element('<div contenteditable="false"></div>')),
    ).toBe(false);
  });
});
