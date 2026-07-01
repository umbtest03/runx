const event_context_raw = process.env.RUNX_INPUT_EVENT_CONTEXT || "{}";
const provided_notes = process.env.RUNX_INPUT_PROVIDED_NOTES || "";
const thread_snippets = process.env.RUNX_INPUT_THREAD_SNIPPETS || "";
const public_link_notes = process.env.RUNX_INPUT_PUBLIC_LINK_NOTES || "";

let event_context = {};
try {
  event_context = JSON.parse(event_context_raw);
} catch(e) {}

if (!event_context.title && !provided_notes && !thread_snippets) {
  process.stdout.write(JSON.stringify({
    needs_input: "Missing context to generate meeting brief."
  }) + "\n");
  process.exit(0);
}

let brief = `# Meeting Brief: ${event_context.title || 'Untitled'}\n`;
if (event_context.date) brief += `**Date:** ${event_context.date}\n`;
if (event_context.attendees) brief += `**Attendees:** ${Array.isArray(event_context.attendees) ? event_context.attendees.join(', ') : event_context.attendees}\n\n`;

brief += `## Synthesis\n`;
if (provided_notes) brief += `- ${provided_notes} [Notes]\n`;
if (thread_snippets) brief += `- ${thread_snippets} [Thread]\n`;
if (public_link_notes) brief += `- ${public_link_notes} [Public Link]\n`;

process.stdout.write(JSON.stringify({
  decision: {
    brief: brief
  }
}) + "\n");
