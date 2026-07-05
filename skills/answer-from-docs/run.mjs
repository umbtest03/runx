const question = process.env.RUNX_INPUT_QUESTION || "";
const corpusRaw = process.env.RUNX_INPUT_CORPUS || "[]";
const corpus = JSON.parse(corpusRaw);

const questionTerms = question.toLowerCase().split(/\s+/).filter(t => t.length > 3);

const citations = [];
const kbGaps = [];

for (const doc of corpus) {
  const docText = doc.text.toLowerCase();
  const matchedTerms = questionTerms.filter(t => docText.includes(t));
  if (matchedTerms.length > 0) {
    citations.push({
      source_id: doc.id,
      excerpt: doc.text.substring(0, 200)
    });
  }
}

if (citations.length > 0) {
  const citedTexts = citations.map(c => c.excerpt);
  process.stdout.write(JSON.stringify({
    answer: {
      text: citedTexts.join(" "),
      citations
    },
    kb_gaps: [],
    grounded: true
  }) + "\n");
} else {
  const gapTerms = questionTerms.filter(t =>
    !corpus.some(d => d.text.toLowerCase().includes(t))
  );
  process.stdout.write(JSON.stringify({
    answer: null,
    kb_gaps: gapTerms,
    grounded: false
  }) + "\n");
  process.exit(3);
}
