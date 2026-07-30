/**
 * Demo data for the marketing screenshots.
 *
 * Every value here is fed to the mocked Tauri IPC layer (see mock-tauri.js), so
 * the screenshots render the real app components against a realistic — but
 * entirely fictional — meeting library. Keep the dates fixed so re-running the
 * capture produces byte-comparable images.
 */

const day = (d, h, m = 0) =>
  `2026-07-${String(d).padStart(2, "0")}T${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:00Z`;

const meeting = (id, title, start, end, status = "summarized") => ({
  id,
  title,
  start_time: start,
  end_time: end,
  audio_path: `/demo/${id}.wav`,
  status,
  calendar_event_id: null,
  raw_notes: null,
  enriched_notes: null,
  template_id: "tpl-standard",
  created_at: start,
  updated_at: end ?? start,
});

const MEETINGS = [
  meeting("m1", "Design Review — Onboarding Flow", day(28, 17, 0), day(28, 17, 38)),
  meeting("m2", "Sprint Planning", day(27, 16, 30), day(27, 17, 17)),
  meeting("m3", "1:1 — Alex Chen", day(27, 20, 0), day(27, 20, 28)),
  meeting("m4", "Customer Discovery — Northwind", day(24, 18, 0), day(24, 19, 2)),
  meeting("m5", "Q3 Roadmap Sync", day(23, 15, 0), day(23, 16, 12)),
  meeting("m6", "Bug Triage", day(22, 22, 0), day(22, 22, 26)),
  meeting("m7", "Interview — Staff Engineer", day(21, 19, 0), day(21, 19, 54)),
  meeting("m8", "Support Sync", day(21, 16, 0), day(21, 16, 24)),
  meeting("m9", "Pricing Workshop", day(20, 17, 30), day(20, 18, 45)),
];

const LABELS = [
  { id: "l1", name: "Product", color: "#8b5cf6", icon: null, created_at: day(1, 9) },
  { id: "l2", name: "Engineering", color: "#22d3ee", icon: null, created_at: day(1, 9) },
  { id: "l3", name: "Customer", color: "#f472b6", icon: null, created_at: day(1, 9) },
  { id: "l4", name: "1:1", color: "#fbbf24", icon: null, created_at: day(1, 9) },
];

const MEETING_LABELS = [
  { meeting_id: "m1", label: LABELS[0] },
  { meeting_id: "m2", label: LABELS[1] },
  { meeting_id: "m3", label: LABELS[3] },
  { meeting_id: "m4", label: LABELS[2] },
  { meeting_id: "m4", label: LABELS[0] },
  { meeting_id: "m5", label: LABELS[0] },
  { meeting_id: "m6", label: LABELS[1] },
  { meeting_id: "m7", label: LABELS[1] },
  { meeting_id: "m8", label: LABELS[2] },
  { meeting_id: "m9", label: LABELS[0] },
];

const line = (id, speaker, text, startMs, endMs) => ({
  id,
  meeting_id: "m1",
  speaker_label: speaker,
  text,
  start_ms: startMs,
  end_ms: endMs,
  confidence: 0.96,
});

const TRANSCRIPT = [
  line("t1", "Speaker 1", "Alright, let's look at where the onboarding rebuild landed. I pulled the funnel numbers this morning.", 12_000, 19_000),
  line("t2", "Speaker 2", "How bad is step three? Last time we looked it was the worst drop in the whole flow.", 21_000, 27_000),
  line("t3", "Speaker 1", "Thirty-one percent drop off on the permissions screen. That's the single biggest leak we have.", 29_000, 36_000),
  line("t4", "Speaker 3", "That tracks with what support is hearing. People don't understand why we ask for screen recording access.", 38_000, 46_000),
  line("t5", "Speaker 2", "So we explain it in place instead of linking out to the docs. One sentence, right above the button.", 48_000, 56_000),
  line("t6", "Speaker 1", "Agreed. Let's ship the inline explainer this sprint and re-measure after a week of data.", 58_000, 66_000),
  line("t7", "Speaker 3", "I can have copy ready by Thursday. I'd like to test two variants if we have the traffic for it.", 68_000, 76_000),
  line("t8", "Speaker 2", "We do. I'll wire up the variant split behind the existing flag so we can turn it off cleanly.", 78_000, 86_000),
  line("t9", "Speaker 1", "Good. The other thing I want to settle today is whether the calendar permission stays in onboarding at all.", 88_000, 97_000),
  line("t10", "Speaker 3", "My vote is we move it. Nobody needs calendar detection in the first sixty seconds.", 99_000, 106_000),
  line("t11", "Speaker 2", "Same. Defer it to the first time someone opens the meetings list with nothing in it.", 108_000, 116_000),
  line("t12", "Speaker 1", "Then that's decided — calendar access moves out of onboarding and becomes contextual.", 118_000, 126_000),
];

const SUMMARY_CONTENT = `## Overview

The team reviewed funnel data for the rebuilt onboarding flow and agreed on two changes: an inline explanation for the screen-recording permission, and deferring the calendar permission out of onboarding entirely.

## Decisions

- **Inline permissions explainer ships this sprint.** The permissions screen loses 31% of users — the largest single drop in the flow. Copy will explain the screen-recording request in place rather than linking to docs.
- **Calendar access leaves onboarding.** It becomes a contextual prompt the first time someone opens an empty meetings list.

## Action items

- Draft two copy variants for the permissions explainer — **Speaker 3**, by Thursday
- Wire the variant split behind the existing feature flag — **Speaker 2**
- Re-measure step-three drop-off after a week of data — **Speaker 1**

## Open questions

- Do we have enough weekly traffic to read a two-variant test inside a sprint?`;

const SUMMARIES = [
  {
    id: "s1",
    meeting_id: "m1",
    template_id: "tpl-standard",
    provider: "anthropic",
    model: "claude-opus-4-6",
    content: SUMMARY_CONTENT,
    created_at: day(28, 17, 40),
  },
];

const INSIGHT_TYPES = [
  {
    id: "it1",
    name: "Decision",
    slug: "decision",
    description: "Choices the group committed to",
    extraction_prompt: "",
    icon: "lightbulb",
    has_action_fields: false,
    is_builtin: true,
    sort_order: 0,
    created_at: day(1, 9),
  },
  {
    id: "it2",
    name: "Action Item",
    slug: "action_item",
    description: "Work someone agreed to pick up",
    extraction_prompt: "",
    icon: "list-checks",
    has_action_fields: true,
    is_builtin: true,
    sort_order: 1,
    created_at: day(1, 9),
  },
  {
    id: "it3",
    name: "Key Moment",
    slug: "key_moment",
    description: "Turning points worth jumping back to",
    extraction_prompt: "",
    icon: "star",
    has_action_fields: false,
    is_builtin: true,
    sort_order: 2,
    created_at: day(1, 9),
  },
  {
    id: "it4",
    name: "Risk",
    slug: "risk",
    description: "Things that could derail the plan",
    extraction_prompt: "",
    icon: "alert-triangle",
    has_action_fields: false,
    is_builtin: false,
    sort_order: 3,
    created_at: day(1, 9),
  },
];

const insight = (o) => ({
  context: null,
  transcript_start_ms: null,
  transcript_end_ms: null,
  action_item_id: null,
  assignee: null,
  due_date: null,
  status: null,
  linear_ticket_id: null,
  action_item_updated_at: null,
  created_at: day(28, 17, 41),
  ...o,
});

const M1_INSIGHTS = [
  insight({
    id: "i1",
    meeting_id: "m1",
    type: "decision",
    content: "Ship an inline explainer for the screen-recording permission this sprint",
    transcript_start_ms: 58_000,
    transcript_end_ms: 66_000,
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i2",
    meeting_id: "m1",
    type: "decision",
    content: "Move calendar access out of onboarding and make it contextual",
    transcript_start_ms: 118_000,
    transcript_end_ms: 126_000,
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i3",
    meeting_id: "m1",
    type: "action_item",
    content: "Draft two copy variants for the permissions explainer",
    action_item_id: "a1",
    assignee: "Speaker 3",
    due_date: "2026-07-30",
    status: "open",
    transcript_start_ms: 68_000,
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i4",
    meeting_id: "m1",
    type: "action_item",
    content: "Wire the variant split behind the existing feature flag",
    action_item_id: "a2",
    assignee: "Speaker 2",
    due_date: "2026-07-31",
    status: "open",
    transcript_start_ms: 78_000,
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i5",
    meeting_id: "m1",
    type: "action_item",
    content: "Re-measure step-three drop-off after a week of data",
    action_item_id: "a3",
    assignee: "Speaker 1",
    due_date: null,
    status: "done",
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i6",
    meeting_id: "m1",
    type: "key_moment",
    content: "31% of users drop off on the permissions screen — the biggest leak in the flow",
    transcript_start_ms: 29_000,
    transcript_end_ms: 36_000,
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
  insight({
    id: "i7",
    meeting_id: "m1",
    type: "risk",
    content: "A two-variant copy test may not reach significance inside one sprint",
    meeting_title: "Design Review — Onboarding Flow",
    meeting_start_time: day(28, 17, 0),
  }),
];

const OTHER_INSIGHTS = [
  insight({
    id: "i8",
    meeting_id: "m2",
    type: "action_item",
    content: "Split the transcription worker refactor into two tickets",
    action_item_id: "a4",
    assignee: "Speaker 2",
    due_date: "2026-07-29",
    status: "open",
    meeting_title: "Sprint Planning",
    meeting_start_time: day(27, 16, 30),
  }),
  insight({
    id: "i9",
    meeting_id: "m4",
    type: "action_item",
    content: "Send Northwind the security questionnaire and SOC 2 summary",
    action_item_id: "a5",
    assignee: "Speaker 1",
    due_date: "2026-07-27",
    status: "done",
    meeting_title: "Customer Discovery — Northwind",
    meeting_start_time: day(24, 18, 0),
  }),
  insight({
    id: "i10",
    meeting_id: "m4",
    type: "decision",
    content: "Northwind pilots with ten seats before the org-wide rollout",
    meeting_title: "Customer Discovery — Northwind",
    meeting_start_time: day(24, 18, 0),
  }),
  insight({
    id: "i11",
    meeting_id: "m5",
    type: "decision",
    content: "Local-only mode ships before the integrations marketplace",
    meeting_title: "Q3 Roadmap Sync",
    meeting_start_time: day(23, 15, 0),
  }),
  insight({
    id: "i12",
    meeting_id: "m5",
    type: "action_item",
    content: "Write the Q3 roadmap post for the changelog",
    action_item_id: "a6",
    assignee: "Speaker 3",
    due_date: "2026-08-03",
    status: "open",
    meeting_title: "Q3 Roadmap Sync",
    meeting_start_time: day(23, 15, 0),
  }),
  insight({
    id: "i13",
    meeting_id: "m6",
    type: "key_moment",
    content: "Crash on resume traced to the audio device change handler",
    meeting_title: "Bug Triage",
    meeting_start_time: day(22, 22, 0),
  }),
];

const ANALYTICS = {
  speakers: [
    {
      id: "sa1",
      meeting_id: "m1",
      speaker_label: "Speaker 1",
      talk_time_ms: 962_000,
      turn_count: 24,
      interruption_count: 1,
      avg_turn_length_ms: 40_083,
      longest_monologue_ms: 118_000,
    },
    {
      id: "sa2",
      meeting_id: "m1",
      speaker_label: "Speaker 2",
      talk_time_ms: 748_000,
      turn_count: 21,
      interruption_count: 3,
      avg_turn_length_ms: 35_619,
      longest_monologue_ms: 96_000,
    },
    {
      id: "sa3",
      meeting_id: "m1",
      speaker_label: "Speaker 3",
      talk_time_ms: 570_000,
      turn_count: 17,
      interruption_count: 2,
      avg_turn_length_ms: 33_529,
      longest_monologue_ms: 87_000,
    },
  ],
  sentiment: [
    { id: "sn1", meeting_id: "m1", start_ms: 0, end_ms: 600_000, sentiment: "neutral", score: 0.12 },
    { id: "sn2", meeting_id: "m1", start_ms: 600_000, end_ms: 1_200_000, sentiment: "positive", score: 0.61 },
    { id: "sn3", meeting_id: "m1", start_ms: 1_200_000, end_ms: 1_800_000, sentiment: "positive", score: 0.48 },
    { id: "sn4", meeting_id: "m1", start_ms: 1_800_000, end_ms: 2_280_000, sentiment: "neutral", score: 0.05 },
  ],
  engagement: {
    id: "e1",
    meeting_id: "m1",
    engagement_level: "high",
    participation_balance: 0.82,
    question_count: 14,
    back_and_forth_ratio: 0.67,
  },
};

const TEMPLATES = [
  {
    id: "tpl-standard",
    name: "Standard Summary",
    description: "Overview, decisions, action items, and open questions",
    sections: "Overview\nDecisions\nAction items\nOpen questions",
    auto_apply_rules: "",
    prompt: "Summarize the meeting transcript into the sections above.",
    is_builtin: true,
    is_favorite: true,
    is_auto_run: true,
    created_at: day(1, 9),
  },
  {
    id: "tpl-standup",
    name: "Daily Standup",
    description: "Yesterday, today, blockers — one block per person",
    sections: "Yesterday\nToday\nBlockers",
    auto_apply_rules: "title contains standup",
    prompt: "Group the transcript by speaker into standup format.",
    is_builtin: true,
    is_favorite: false,
    is_auto_run: true,
    created_at: day(1, 9),
  },
  {
    id: "tpl-1on1",
    name: "1:1 Notes",
    description: "Themes, feedback, growth, and follow-ups",
    sections: "Themes\nFeedback\nGrowth\nFollow-ups",
    auto_apply_rules: "title contains 1:1",
    prompt: "Summarize this one-on-one, keeping feedback verbatim where possible.",
    is_builtin: true,
    is_favorite: true,
    is_auto_run: false,
    created_at: day(1, 9),
  },
  {
    id: "tpl-customer",
    name: "Customer Call",
    description: "Pain points, requests, objections, and next steps",
    sections: "Pain points\nFeature requests\nObjections\nNext steps",
    auto_apply_rules: "",
    prompt: "Extract customer signal from the transcript.",
    is_builtin: false,
    is_favorite: false,
    is_auto_run: false,
    created_at: day(4, 11),
  },
  {
    id: "tpl-interview",
    name: "Interview Debrief",
    description: "Signals, concerns, and a hire recommendation",
    sections: "Signals\nConcerns\nRecommendation",
    auto_apply_rules: "title contains interview",
    prompt: "Write an interview debrief from the transcript.",
    is_builtin: false,
    is_favorite: false,
    is_auto_run: false,
    created_at: day(9, 14),
  },
];

const RECIPES = [
  {
    id: "r1",
    name: "Follow-up Email",
    description: "Draft a follow-up email to everyone on the call",
    slash_command: "/email",
    prompt_template: "Write a short follow-up email covering the decisions and action items.",
    output_format: "markdown",
    is_builtin: true,
    created_at: day(1, 9),
    updated_at: day(1, 9),
  },
  {
    id: "r2",
    name: "Decision Log",
    description: "List every decision with the reasoning behind it",
    slash_command: "/decisions",
    prompt_template: "List each decision made, with who raised it and why it was chosen.",
    output_format: "markdown",
    is_builtin: true,
    created_at: day(1, 9),
    updated_at: day(1, 9),
  },
  {
    id: "r3",
    name: "Standup Update",
    description: "Turn the call into a three-line standup post",
    slash_command: "/standup",
    prompt_template: "Summarize this meeting as a three-line standup update.",
    output_format: "markdown",
    is_builtin: false,
    created_at: day(12, 10),
    updated_at: day(12, 10),
  },
];

const INTEGRATIONS = [
  { id: "int1", integration_type: "slack", name: "Acme Workspace", credentials_json: "{}", created_at: day(2, 9) },
  { id: "int2", integration_type: "notion", name: "Product Wiki", credentials_json: "{}", created_at: day(2, 9) },
  { id: "int3", integration_type: "linear", name: "Linear — Product", credentials_json: "{}", created_at: day(3, 9) },
];

const WORKFLOWS = [
  {
    id: "w1",
    name: "Post summary to Slack",
    description: "Sends the generated summary to #product-updates",
    icon: "💬",
    integration_id: "int1",
    action_type: "slack_message",
    config_json: '{"channel":"#product-updates"}',
    is_enabled: true,
    created_at: day(2, 9),
  },
  {
    id: "w2",
    name: "Append notes to Notion",
    description: "Adds the meeting to the Product Wiki database",
    icon: "📝",
    integration_id: "int2",
    action_type: "notion_page",
    config_json: '{"database":"Meetings"}',
    is_enabled: true,
    created_at: day(2, 9),
  },
  {
    id: "w3",
    name: "Create Linear issues",
    description: "Opens an issue for every open action item",
    icon: "🎫",
    integration_id: "int3",
    action_type: "linear_issue",
    config_json: '{"team":"PROD"}',
    is_enabled: true,
    created_at: day(3, 9),
  },
];

const WORKFLOW_RUNS = [
  {
    id: "wr1",
    meeting_id: "m1",
    workflow_id: "w1",
    status: "completed",
    result_json: '{"permalink":"#product-updates"}',
    error: null,
    started_at: day(28, 17, 42),
    completed_at: day(28, 17, 42),
    workflow_name: "Post summary to Slack",
    workflow_icon: "💬",
  },
  {
    id: "wr2",
    meeting_id: "m1",
    workflow_id: "w2",
    status: "completed",
    result_json: '{"page":"Design Review — Onboarding Flow"}',
    error: null,
    started_at: day(28, 17, 42),
    completed_at: day(28, 17, 43),
    workflow_name: "Append notes to Notion",
    workflow_icon: "📝",
  },
  {
    id: "wr3",
    meeting_id: "m1",
    workflow_id: "w3",
    status: "completed",
    result_json: '{"issues":2}',
    error: null,
    started_at: day(28, 17, 43),
    completed_at: day(28, 17, 43),
    workflow_name: "Create Linear issues",
    workflow_icon: "🎫",
  },
];

const CONVERSATIONS = [
  { id: "c1", title: "Onboarding drop-off", created_at: day(28, 18, 0), updated_at: day(28, 18, 6) },
  { id: "c2", title: "Northwind commitments", created_at: day(25, 9, 0), updated_at: day(25, 9, 12) },
  { id: "c3", title: "Open action items", created_at: day(24, 8, 30), updated_at: day(24, 8, 34) },
];

const CHAT_MESSAGES = {
  c1: [
    {
      id: "cm1",
      conversation_id: "c1",
      role: "user",
      content: "What have we said about onboarding drop-off in the last month?",
      sources_json: null,
      created_at: day(28, 18, 0),
    },
    {
      id: "cm2",
      conversation_id: "c1",
      role: "assistant",
      content: `Onboarding drop-off has come up in three meetings:

**Design Review — Onboarding Flow (Jul 28)** — The permissions screen loses **31%** of users, the largest single drop in the flow. The team decided to explain the screen-recording request inline instead of linking to docs, and to move calendar access out of onboarding entirely.

**Sprint Planning (Jul 27)** — The explainer was pulled into this sprint. The variant split rides on the existing feature flag so it can be turned off without a release.

**Q3 Roadmap Sync (Jul 23)** — Onboarding was ranked behind local-only mode but ahead of the integrations marketplace.

The one open question: whether a two-variant copy test can reach significance inside a single sprint.`,
      sources_json: JSON.stringify([
        { meeting_id: "m1", meeting_title: "Design Review — Onboarding Flow", start_ms: 29_000, end_ms: 36_000 },
        { meeting_id: "m1", meeting_title: "Design Review — Onboarding Flow", start_ms: 118_000, end_ms: 126_000 },
        { meeting_id: "m2", meeting_title: "Sprint Planning", start_ms: 412_000, end_ms: 425_000 },
        { meeting_id: "m5", meeting_title: "Q3 Roadmap Sync", start_ms: 1_204_000, end_ms: 1_219_000 },
      ]),
      created_at: day(28, 18, 1),
    },
    {
      id: "cm3",
      conversation_id: "c1",
      role: "user",
      content: "Who owns the follow-up work?",
      sources_json: null,
      created_at: day(28, 18, 5),
    },
    {
      id: "cm4",
      conversation_id: "c1",
      role: "assistant",
      content: `Three owners came out of the design review:

- **Speaker 3** — two copy variants for the permissions explainer, due Thursday
- **Speaker 2** — the variant split behind the existing feature flag
- **Speaker 1** — re-measuring step-three drop-off after a week of data

Speaker 1's item is already marked done.`,
      sources_json: JSON.stringify([
        { meeting_id: "m1", meeting_title: "Design Review — Onboarding Flow", start_ms: 68_000, end_ms: 86_000 },
      ]),
      created_at: day(28, 18, 6),
    },
  ],
  c2: [],
  c3: [],
};

const MODELS = [
  { id: "claude-opus-4-6", name: "Claude Opus 4.6", provider: "anthropic" },
  { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6", provider: "anthropic" },
  { id: "gpt-5.2", name: "GPT-5.2", provider: "openai" },
  { id: "gemini-3-pro", name: "Gemini 3 Pro", provider: "google" },
  { id: "llama3.3:70b", name: "Llama 3.3 70B", provider: "ollama" },
  { id: "qwen3:14b", name: "Qwen 3 14B", provider: "ollama" },
];

const PROVIDERS = ["anthropic", "openai", "google", "groq", "openrouter", "bedrock", "ollama"];

/**
 * A one-second silent WAV. The meeting detail player needs a real source to
 * render as loaded; capture.mjs overrides the reported duration so the
 * transport matches the demo meeting's length.
 */
function silentWav() {
  const rate = 8000;
  const samples = rate;
  const header = Buffer.alloc(44);
  header.write("RIFF", 0);
  header.writeUInt32LE(36 + samples, 4);
  header.write("WAVE", 8);
  header.write("fmt ", 12);
  header.writeUInt32LE(16, 16);
  header.writeUInt16LE(1, 20); // PCM
  header.writeUInt16LE(1, 22); // mono
  header.writeUInt32LE(rate, 24);
  header.writeUInt32LE(rate, 28);
  header.writeUInt16LE(1, 32);
  header.writeUInt16LE(8, 34);
  header.write("data", 36);
  header.writeUInt32LE(samples, 40);
  return Buffer.concat([header, Buffer.alloc(samples, 128)]).toString("base64");
}

/** Length of the demo meeting, in seconds — see silentWav(). */
export const AUDIO_DURATION_SECONDS =
  (new Date(MEETINGS[0].end_time) - new Date(MEETINGS[0].start_time)) / 1000;

/** Transcript pushed into the live recording view via `transcript-update`. */
export const LIVE_TRANSCRIPT = TRANSCRIPT.slice(0, 6).map((s) => ({
  ...s,
  meeting_id: "m-live",
}));

/**
 * Command name -> response. Anything not listed here resolves to `null`, which
 * is enough for the fire-and-forget commands the screenshots never trigger.
 */
export const FIXTURES = {
  "plugin:app|version": "0.1.1",
  "plugin:window|inner_size": { width: 1440, height: 900 },
  "plugin:window|current_monitor": {
    name: "Built-in Retina Display",
    size: { width: 3024, height: 1964 },
    position: { x: 0, y: 0 },
    scaleFactor: 2,
    workArea: { position: { x: 0, y: 0 }, size: { width: 3024, height: 1890 } },
  },

  list_meetings: MEETINGS,
  get_meeting: MEETINGS[0],
  get_transcript: TRANSCRIPT,
  get_summaries: SUMMARIES,
  get_insights: M1_INSIGHTS,
  get_all_insights: [...M1_INSIGHTS, ...OTHER_INSIGHTS],
  list_insight_types: INSIGHT_TYPES,
  get_meeting_analytics: ANALYTICS,
  get_audio_data: silentWav(),
  get_scratch_notes: [],

  list_labels: LABELS,
  get_all_meeting_labels: MEETING_LABELS,
  get_meeting_labels: [LABELS[0]],

  list_templates: TEMPLATES,
  list_recipes: RECIPES,
  list_integrations: INTEGRATIONS,
  list_workflows: WORKFLOWS,
  list_workflow_runs: WORKFLOW_RUNS,

  list_chat_conversations: CONVERSATIONS,
  list_chat_messages: CHAT_MESSAGES,
  get_embedding_status: { embedded: 6, total: 6, model_available: true },

  list_llm_models: MODELS,
  list_llm_providers: PROVIDERS,
  list_stored_providers: ["anthropic", "openai", "ollama"],
  has_api_key: true,
  get_downloaded_models: ["parakeet-v3", "nomic-embed-text"],
  get_available_models: [],
  get_app_setting: null,
  check_permissions: { microphone: true, screen_recording: true, calendar: true },

  get_linear_tickets: [],
  list_linear_teams: [],
  list_linear_projects: [],
  get_linear_setting: null,

  is_recording: false,
  start_recording: meeting(
    "m-live",
    "Design Review — Onboarding Flow",
    day(28, 17, 0),
    null,
    "recording",
  ),
};
