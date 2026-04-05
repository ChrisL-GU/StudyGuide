# Building StudyGuide with a Coding Agent

This document contains a series of prompts you can give to a coding agent (such as [Claude Code](https://claude.ai/claude-code)) to recreate the StudyGuide project from scratch. Feed each prompt one at a time, wait for the agent to finish, verify the result, then move to the next.

The prompts are ordered to build the project incrementally — each step builds on the previous one and produces a working (if incomplete) application.

## Prerequisites

Before starting, make sure you have installed:

- Node.js 18+
- Rust (via rustup)
- Tauri system dependencies for your platform (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))
- `cmake`, `gcc-c++`/`g++`, and `clang-devel`/`libclang-dev` (needed for compiling whisper.cpp)

You should also have:

- An OpenAI-compatible API key and endpoint (OpenAI, Azure OpenAI, or similar)
- Optionally, an OpenAI-compatible TTS API endpoint for voice output

---

## Prompt 1: Project Scaffolding

> Create a new Tauri 2 desktop application using SvelteKit as the frontend framework with TypeScript. Use the static adapter for SvelteKit since this is a single-page app inside a desktop shell. Configure the Vite dev server on port 1420 for Tauri compatibility. The app should be called "StudyGuide" with identifier "com.studyguide.app". Verify it compiles and the empty window opens with `npx tauri dev`.

## Prompt 2: Layout and Navigation

> Create a sidebar navigation layout for the app with three pages: Documents, Chat, and Settings. The sidebar should be on the left with the app name "StudyGuide" at the top and links to each page below. Use SvelteKit file-based routing with `+page.svelte` files for each route. The root page (`/`) should redirect to `/documents`. Add placeholder content to each page for now. Use a dark theme with a navy/slate color palette.

## Prompt 3: Settings Persistence

> Create a Settings page and backend that lets users configure an OpenAI-compatible API. The settings form should have fields for: API Key, Base URL (default "https://api.openai.com/v1"), Model name (default "gpt-4o"), and API Version (leave blank for standard OpenAI, fill in for Azure). On the Rust side, create a `settings.rs` module that stores settings as a JSON file in the Tauri app data directory. Create Tauri commands `get_settings` and `save_settings`, and register them. On the frontend, create a TypeScript types file with an `AppSettings` interface, a `tauri.ts` module with typed wrappers around `invoke`, and a reactive store using Svelte 5 runes to hold the settings state. Load saved settings on mount and save them when the user clicks Save.

## Prompt 4: Document Parsing

> Add the ability to load and parse documents. Support four formats: PDF, DOCX, TXT, and Markdown. Create a `parser.rs` module in the Rust backend. For TXT and MD files, read the file contents directly. For PDF, use the `pdf-extract` crate. For DOCX, read the file as a ZIP archive using the `zip` crate, extract `word/document.xml`, and parse the XML with `quick-xml` to extract text content from `<w:t>` elements while maintaining paragraph breaks. Create a `Document` struct with id (UUID), name (filename), and content fields. Create a `DocumentStore` that holds loaded documents in memory, managed as `Mutex<DocumentStore>` state in Tauri. Add Tauri commands: `load_document` (takes a file path, parses it, stores it, returns the doc without content to keep the payload small), `list_documents` (returns all docs without content), and `remove_document` (removes by id).

## Prompt 5: Document Management UI

> Create the Documents page UI. It should have an "Add Document" button that opens a native file picker dialog (using `tauri-plugin-dialog`) filtered to PDF, DOCX, TXT, and MD files. Display loaded documents as a list of cards, each with a checkbox to select/deselect it for chat context, the filename, a file type badge, and a remove button. Store the selected document IDs in a reactive `Set<string>` in the shared store. Load the document list from the backend on mount. The add/remove operations should update both the backend and frontend state.

## Prompt 6: Chat API Integration

> Create the Chat page and wire it up to an OpenAI-compatible API. On the Rust side, create an `api.rs` module with a `chat_completion` function. It should detect Azure endpoints by checking if `api_version` is non-empty — for Azure, build the URL as `{base_url}/openai/deployments/{model}/chat/completions?api-version={api_version}` and use an `api-key` header; for standard OpenAI, append `/chat/completions` to the base URL and use a Bearer token. Build a system prompt that instructs the AI to be an intellectual reading companion: it should walk through study materials section by section, ask what the reader thinks rather than explaining, surface hidden assumptions, play devil's advocate, challenge incomplete reasoning, and connect ideas across sections. Include the full text of selected documents in the system prompt. Create a `chat` Tauri command that takes the message history and selected document IDs, builds the full prompt, and calls the API. On the frontend, create a chat interface with a message list, text input, and send button. Enter should send, Shift+Enter for newline. Display user and assistant messages with distinct styling.

## Prompt 7: Streaming Responses

> Update the chat to stream responses instead of waiting for the full reply. On the Rust side, add `stream: true` to the API request body, add the `stream` feature to the `reqwest` dependency and add `futures-util`. Read the response as a byte stream, parse Server-Sent Events line by line, extract content deltas from `choices[0].delta.content` in each `data:` line, and emit each chunk to the frontend as a `chat-stream-chunk` Tauri event. On the frontend, listen for these events using `@tauri-apps/api/event`, and update the assistant message content in real-time as chunks arrive. Show a typing indicator (animated dots) before the first token arrives, then replace it with the streaming text.

## Prompt 8: Retry Logic

> Add retry logic to the chat API call. Network errors (connection failures, timeouts) should retry up to 3 times with increasing delay (0ms, 500ms, 1000ms). Pre-serialize the request body to a byte vector so each retry can clone it and build a fresh request. Do not retry on API errors (4xx/5xx responses) — only on send failures.

## Prompt 9: Speech-to-Text with Whisper

> Add local speech-to-text using Whisper. On the Rust side, add the `whisper-rs` crate (v0.13). Create a `voice.rs` module with: a `WhisperState` struct holding an `Option<WhisperContext>` (for lazy-loading), a function to check if the Whisper model is downloaded, a function to download the `ggml-base.en.bin` model (~142MB) from HuggingFace into the app data directory, and a `transcribe_audio` function that lazy-loads the model on first use, converts incoming bytes (little-endian f32) to samples, and runs Whisper inference with greedy sampling, English language, suppressing blanks and non-speech tokens. Add Tauri commands: `get_voice_model_status`, `download_voice_models`, and `transcribe`. Manage `WhisperState` as `Mutex<WhisperState>` in the Tauri app state. On the frontend, create an `audio.ts` module that records from the microphone using `AudioContext` and `ScriptProcessorNode` (not MediaRecorder — WebKitGTK can't decode the webm it produces) at 16kHz mono, collecting raw PCM samples. On stop, concatenate all chunks into a Float32Array and convert to little-endian f32 bytes. Add a mic button to the chat input area. On the Settings page, show the Whisper model download status and a download button. Create a `voiceState` module with reactive getters/setters for `isRecording`, `isTranscribing`, `isSpeaking`, `ttsEnabled`, and `modelsReady`.

## Prompt 10: Microphone Permissions on Linux

> On Linux, Tauri uses WebKitGTK which blocks `getUserMedia` by default. In the Tauri setup hook in `lib.rs`, add a Linux-specific block that gets the main webview window and connects to the WebKit permission-request signal, auto-allowing all permission requests. Use `webkit2gtk` as a Linux-only dependency (`[target.'cfg(target_os = "linux")'.dependencies]`) with the `PermissionRequestExt` and `WebViewExt` traits.

## Prompt 11: Cloud Text-to-Speech

> Add text-to-speech using an OpenAI-compatible TTS API (not a local engine). Add settings fields for TTS: `tts_api_key`, `tts_base_url`, `tts_api_version` (all optional, falling back to the main API settings if blank), `tts_model` (default "tts"), and `tts_voice` (default "alloy"). Create a `text_to_speech` function in `api.rs` that POSTs to the `/audio/speech` endpoint (or Azure equivalent) with model, input text, voice, and `response_format: "wav"`, returning the raw WAV bytes. Update the `speak` Tauri command to use this cloud API. On the frontend, add a TTS toggle button in the chat header that enables auto-speaking of AI responses. Add a small speaker button on each assistant message for manual replay. Add the TTS settings fields to the Settings page with a collapsible section for the TTS-specific endpoint override. Add a voice selector dropdown with options: alloy, echo, fable, nova, onyx, shimmer. In `audio.ts`, add `playWavBytes` (decodes WAV and plays via AudioContext) and `stopPlayback` functions.

## Prompt 12: Keyboard Shortcuts

> Add keyboard shortcuts to the chat page. Hold Space (when not focused in the textarea) to start recording, release to stop and transcribe. Enter (when not focused in the textarea) sends the message — this makes the flow: hold Space to dictate, release, press Enter to send without needing the mouse. Keep the existing Enter-to-send behavior inside the textarea. Use global `keydown`/`keyup` event listeners added in `onMount` and cleaned up in `onDestroy`. Track a `spaceHeld` flag to prevent key-repeat from triggering multiple recordings.

## Prompt 13: Input Polish

> Make the chat textarea auto-grow as the user types or when text is inserted by voice transcription. On the `input` event, reset the textarea height to `auto` then set it to `scrollHeight`. Also call this resize function via `requestAnimationFrame` after transcription sets the input value and after sending clears it. Set a `max-height` of around 120px so it doesn't take over the screen. When voice transcription produces text and there's already text in the input, append the new text with two newlines rather than overwriting.

## Prompt 14: Collapsible Sidebar

> Make the sidebar collapsible. Add a chevron button at the bottom of the sidebar. When collapsed, the sidebar should shrink to icon-only width (~52px), hiding the text labels but still showing navigation icons. The nav links should have `title` attributes so they show tooltips on hover when collapsed. Add a smooth CSS transition on the width change. The chevron should rotate 180 degrees when collapsed.

## Prompt 15: Visual Polish

> Redesign the entire UI to be polished and modern. Use a dark theme throughout with a deep navy base (#0c0f1a), slate grays for surfaces, and indigo (#6366f1) as the accent color. Specific touches: gradient brand text in the sidebar, SVG icons for all navigation and actions (no emoji), custom thin scrollbars, subtle hover animations (translateY lifts on buttons, opacity reveals on action buttons), animated typing indicator with bouncing dots, a unified input bar in chat with embedded mic and send buttons, avatar icons for user (person) and tutor (book), organized settings sections with icons, loading spinners, focus rings with box-shadow, and smooth transitions throughout. The UI should feel like a professional desktop application.

---

## Notes for the builder

- **Test incrementally.** Run `npx tauri dev` after each prompt to verify things work before moving on.
- **Azure OpenAI specifics.** If using Azure, the base URL is just the resource endpoint (e.g., `https://your-resource.openai.azure.com`), the model field is the deployment name, and you must set the API version. The TTS deployment may be on a different resource with a different key — that's why the TTS settings are separate.
- **Whisper build dependencies.** The `whisper-rs` crate compiles whisper.cpp from source and needs `cmake`, a C++ compiler, and `libclang`. Install these before Prompt 9 or the build will fail.
- **Prompt order matters.** The prompts build on each other. Skipping one will likely cause later prompts to fail or produce unexpected results.
- **Adapt to your agent.** Different coding agents may need more or less detail. If a prompt produces errors, give the agent the error message and let it fix it before moving on. You may need to split complex prompts or give additional context.
- **Make it your own.** These prompts produce one version of StudyGuide. Feel free to modify the system prompt, change the color scheme, add features, or take the project in a different direction.
