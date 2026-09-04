# Design mockups — index

Every file in this directory is **current ratified design as of NA-0703
(desktop D-0028)**. Superseded drafts are deliberately NOT kept here — they
live in this path's git history and in the operator-side archive. If you are
asking "which mockup is current?", the answer is: whatever this directory
contains, as indexed below.

## The authority rule

- **Layout and structure authority ONLY.** A mockup fixes what a screen
  contains, how it is arranged, and what the copy says.
- **Mockups are NOT shipped UI.** The shipped markup lives in `ui/`; a mockup
  and the app may lag each other — the mockup is the design of record, not a
  screenshot.
- **Colour comes from the shipped design tokens**
  (`docs/DESIGN_SPEC_AppendixF.md`). A mockup's palette is illustrative;
  reading a hex out of a mockup into the build is a STOP.
- All names, message text, fingerprints, invite codes, addresses, and paths in
  these files are fabricated placeholders; the sanitization record rides the
  NA-0703 lane record.

## Naming

Numbered screens use `mockup-NN-slug.html` (the dominant measured form;
`06e-server-pane.html` / `06e2-server-pane-no-token.html` keep their
historical names — existing files are not renamed). `MOCKUP_*` files are
named design/ratification artifacts and keep their ruled names because other
records cite them by filename.

## Index

| file | depicts | status |
|---|---|---|
| `06e-server-pane.html` | Settings › Server (relay) pane, populated state | current (lineage D610/D-0010; body refreshed NA-0703) |
| `06e2-server-pane-no-token.html` | Settings › Server pane, no-token state | current (lineage D610/D-0010; body refreshed NA-0703) |
| `mockup-07-identity-pane.html` | Settings › Identity pane | current (fingerprint form per two-tier ratification 2026-08-01) |
| `mockup-07b-onboarding-identity.html` | Onboarding: "This is you" step | current (fingerprint form per two-tier ratification 2026-08-01) |
| `mockup-08-create-vault.html` | Onboarding: create your vault | current |
| `mockup-08b-create-vault-suggest.html` | Create vault, suggested-passphrase shown state | current (companion to 08) |
| `mockup-09-vault-security-pane.html` | Settings › Vault & Security pane (auto-lock, erase-after-failures, destroy) | current |
| `mockup-10-focus-ring.html` | Focus-ring treatment comparison | current |
| `mockup-11-main-chat-view.html` | Main chat view — three-pane, conversation open | current (chat era) |
| `mockup-12-unverified-thread.html` | Thread with an unverified contact | current (chat era) |
| `mockup-13-verify-contact.html` | Verify a contact — compare fingerprints (standalone) | current (chat era) |
| `mockup-13a-verify-modal.html` | Verify as a modal overlay | current (chat era) |
| `mockup-14-invite-create.html` | Create an invite (modal, steps 1–3 + reject) | current (chat era) |
| `mockup-15-add-contact.html` | Add a contact (redeem invite) | current (chat era) |
| `mockup-16-invitations-page.html` | Settings › Invitations — sent and received invitations with their state; the Contacts-pane entry point (superseded by 17) | blessed 2026-08-31 (v6); layout authority for NA-0778; its "Connected" states and its Clear-on-expired action have no engine source at 0b87209b — see desktop `D-0047` |
| `mockup-17-contacts-pane-invitations-block.html` | Contacts pane: the non-clickable "Invitations" label with review · redeem · send | blessed 2026-09-01 (v4); entry-point authority for NA-0778; supersedes the single-link entry point of mockup 16's note 1 |
| `MOCKUP_channel_established_verify_banner.html` | Channel-established verify banner, States 0/1/2 | ratified 2026-08-01; success path only — failure states in the companion below |
| `MOCKUP_channel_establish_FAILURE_STATES.html` | Channel establishment failure states S-F1..S-F5 | ruled 2026-08-08; companion to the banner mockup |
| `MOCKUP_fingerprint_two_tier_RATIFIED.html` | Fingerprint two-tier display: 30-digit voice form + 256-bit hex | ratified 2026-08-01; format reference for 07/07b/13/13a |
