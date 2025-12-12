# Phoenix Desktop (Tauri) — Multi‑Modal Recording UI

This is a **minimal scaffold** for a Phoenix desktop UI using **Tauri v2**.

Backend commands are implemented in [`phoenix-desktop-tauri/src-tauri/src/main.rs`](phoenix-desktop-tauri/src-tauri/src/main.rs:1) and call into [`multi_modal_recording::MultiModalRecorder`](multi_modal_recording/src/lib.rs:56).

## Intended UI

- Buttons:
  - Record Audio
  - Record Video
  - Record Audio+Video
  - Schedule
  - Always Listening toggle
  - Enrollment wizard (voice + face)
- Status:
  - Live preview placeholder
  - Recognition label (e.g. `I see you, Dad ❤️`)

## Notes

- The default workspace build keeps multi-modal capture/recognition behind feature flags.
- To turn this into a full desktop app, generate a frontend (Vanilla/React/Svelte) and wire it to Tauri `invoke()` calls.

