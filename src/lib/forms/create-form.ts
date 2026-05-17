import { createForm as felteCreateForm } from "felte";
import { validator } from "@felte/validator-zod";
import type { FormConfigWithoutTransformFn } from "@felte/core";
import type { ZodTypeAny, z } from "zod";

type ZodData<Schema extends ZodTypeAny> = z.infer<Schema> & Record<string, unknown>;

/**
 * Wraps felte's `createForm` with a zod schema as the source of truth for
 * shape, validation, and inferred TS types. Feature panels supply a schema
 * and an optional felte config and get back a fully wired form.
 */
export function createForm<Schema extends ZodTypeAny>(
  schema: Schema,
  config: Omit<FormConfigWithoutTransformFn<ZodData<Schema>>, "extend" | "validate"> = {},
) {
  return felteCreateForm<ZodData<Schema>>({
    ...config,
    extend: [validator({ schema })],
  });
}
