# 68 — OAuth2 Apps: Image Tab

The Image tab on the OAuth2 detail page. Upload, preview, or remove the application's icon image shown on the kanidm apps listing.

## Purpose

Manage the OAuth2 client's image attribute. The image appears on the kanidm self-service apps page (the list users see when they navigate to `/ui/apps`). Supported formats per kanidm: PNG, JPG, SVG, WEBP, GIF.

## API endpoints

- **Display image:** `GET /ui/images/oauth2/{name}` — returns the raw image bytes with proper `Content-Type` header. Keyed by **client_id (name)**, NOT by the SHA256 hash in the entry's `image` attr. The UI loads the image via this URL directly (`<img src="/ui/images/oauth2/grafana">`).
- **Upload:** `POST /v1/oauth2/{name}/_image` — multipart/form-data with a field named `image` containing the file (filename, bytes, content-type).
- **Delete:** `DELETE /v1/oauth2/{name}/_image`.
- **Existence check:** the entry's `image` attr is present iff an image has been uploaded. The attr value is an internal hash; the UI does not need to use it directly.

## Layout

Tab content inside OAuth2 detail page:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Image                                                               │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Current image                                                   │ │
│ │                                                                 │ │
│ │     ┌────────────┐                                              │ │
│ │     │            │                                              │ │
│ │     │ [Grafana   │                                              │ │
│ │     │  icon]     │                                              │ │
│ │     │            │                                              │ │
│ │     └────────────┘                                              │ │
│ │     128 × 128 px                                                │ │
│ │     grafana.svg                                                 │ │
│ │     12 KB                                                       │ │
│ │                                                                 │ │
│ │     [Replace image] [Remove]                                    │ │
│ └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│ ┌─────────────────────────────────────────────────────────────────┐ │
│ │ Upload guidelines                                               │ │
│ │                                                                 │ │
│ │  ▸ Formats: PNG, JPG, SVG, WEBP, GIF                            │ │
│ │  ▸ Recommended size: at least 128 × 128 pixels (square)         │ │
│ │  ▸ Maximum file size: 256 KB                                    │ │
│ │  ▸ Background: transparent preferred (for SVG/PNG/WEBP)         │ │
│ │  ▸ Used on the apps listing and consent screens                 │ │
│ └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Tab content

### Current image card

Two states:

**State A — Image set:**
- Preview (128-160px box, centered or left-aligned per design)
- Metadata below preview:
  - Dimensions ("128 × 128 px")
  - Filename ("grafana.svg")
  - File size ("12 KB")
- Actions:
  - "Replace image" — opens upload (same as Upload below). Picks new file → uploads → preview refreshes.
  - "Remove" — danger-secondary. Confirm: "Remove the image? The default placeholder will be used on the apps listing."

**State B — No image set:**
- Empty preview state (placeholder rectangle with icon + "No image uploaded")
- Primary action: "Upload image" — opens file picker

### Upload guidelines card

Static informational card listing the upload constraints:
- Formats accepted
- Recommended size
- Max file size
- Transparency note
- Where it's displayed

## Upload flow

Clicking "Upload image" or "Replace image" opens the native file picker (filtered to image MIME types).

After selection, the UI:
1. Validates client-side (size, format)
2. Shows a small preview of the selected file with the filename and size
3. Provides Confirm Upload + Cancel buttons

```
   ┌──────────────────────────────────────────────────┐
   │  Upload image                              [×]   │
   ├──────────────────────────────────────────────────┤
   │                                                  │
   │     ┌────────────┐                               │
   │     │            │                               │
   │     │ [Preview]  │                               │
   │     │            │                               │
   │     └────────────┘                               │
   │     grafana-new.svg                              │
   │     14 KB                                        │
   │                                                  │
   │     ☑ This will replace the current image.       │
   │                                                  │
   ├──────────────────────────────────────────────────┤
   │              [Cancel]       [Upload]             │
   └──────────────────────────────────────────────────┘
```

On Upload: calls `POST /v1/oauth2/{name}/_image` with multipart form data. Toast on success. Refresh preview.

## States

- **Loading current image:** skeleton placeholder.
- **State A (image set):** as described.
- **State B (no image):** empty preview + upload CTA.
- **Picking file:** native picker.
- **Confirming upload:** confirm modal/state with preview.
- **Uploading:** progress indicator on the Upload button.
- **Upload success:** preview updates, toast.
- **Upload failure (size too large, wrong format, server reject):** inline error in confirm modal: "File is too large (256 KB max)." or "Unsupported format. Use PNG, JPG, SVG, WEBP, or GIF."

## Sample data

For Grafana (has image):
- Current image: a Grafana logo (sample SVG / PNG)
- Metadata: 128×128, `grafana.svg`, 12 KB

For Vaultwarden (no image): State B — empty preview, upload CTA.

For the upload-confirm mockup, show a new file `grafana-new.svg` with 14 KB.

## Edge cases

- **SVG with embedded scripts:** kanidm should sanitize. UI shows the image regardless.
- **Animated GIF:** allowed but rendered as static thumbnail in admin UI (or first frame).
- **Very small image (e.g., 16×16):** allowed but warn: "⚠ The image is smaller than recommended. It may appear blurry on the apps listing."
- **Very wide / very tall (non-square):** allowed; preview shows aspect ratio honestly.
- **Re-uploading the same file:** allowed; just replaces.
- **Privilege required:** upload/remove requires privilege session.

## Mockup elements to render

- Tab content with "Image" heading
- Current image card (State A) for Grafana: preview, metadata, Replace + Remove buttons
- Upload guidelines card with all bullets
- Render State B for Vaultwarden (no image, upload CTA)
- Render the Upload confirm modal with a new file preview
- Render a Remove confirm modal
