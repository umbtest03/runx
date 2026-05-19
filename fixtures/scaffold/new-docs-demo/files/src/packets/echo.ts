import { definePacket, t } from "@runxhq/authoring";

export const EchoPacket = definePacket({
  id: "docs.demo.echo.v1",
  schema: t.Object({
    message: t.String(),
  }),
});
