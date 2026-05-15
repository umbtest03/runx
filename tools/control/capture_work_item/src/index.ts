import { defineTool, isRecord, recordInput } from "@runxhq/authoring";

export default defineTool({
  name: "control.capture_work_item",
  description: "Capture the current work-item packet as an explicit graph context value.",
  inputs: {
    work_item: recordInput({ optional: true, description: "Optional runx.work_item.v1 packet from the issue control plane." }),
  },
  scopes: ["runx:control:read"],
  run({ inputs }) {
    const workItem = isRecord(inputs.work_item) ? inputs.work_item : { captured: false };
    return {
      present: isRecord(inputs.work_item),
      work_item: workItem,
    };
  },
});
