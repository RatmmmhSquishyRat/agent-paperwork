# Agent Paperwork — BDD Scenarios

## Feature: Workspace Initialization

### Scenario: First agent bootstraps a workspace
```gherkin
Given an empty directory
When I run `paperwork init --name alice --model gpt-4o`
Then a `.paperwork/` directory exists
And `.paperwork/profiles/alice.md` contains "# alice" and "**Model**: gpt-4o"
And `.paperwork/contacts.md` contains a table row for alice
And `.paperwork/dm/`, `.paperwork/posts/`, `.paperwork/manifests/`, `.paperwork/notifications/` directories exist
```

### Scenario: Init is idempotent
```gherkin
Given a workspace already initialized with agent alice
When I run `paperwork init --name alice`
Then no error occurs
And existing files are not modified
```

---

## Feature: Profile Management

### Scenario: Create a profile with stub-first UX
```gherkin
Given an initialized workspace
When I run `paperwork profile create bob --model claude-4`
Then `.paperwork/profiles/bob.md` exists with "# bob" and "**Model**: claude-4"
And `.paperwork/contacts.md` now includes bob
And scope fields default to "—" (empty)
```

### Scenario: Edit profile scope
```gherkin
Given profile bob exists
When I run `paperwork profile edit bob --scope-read "src/**" --scope-owns "src/parser/**"`
Then bob's profile shows **Read**: `src/**` and **Owns**: `src/parser/**`
```

### Scenario: Show profile
```gherkin
Given profile alice exists with model gpt-4o and scope owns src/parser/**
When I run `paperwork profile show alice`
Then output displays name, model, description, and all scope fields
```

### Scenario: List all profiles
```gherkin
Given profiles alice and bob exist
When I run `paperwork profile list`
Then output lists both alice and bob
```

---

## Feature: Invite & Contacts

### Scenario: Invite creates profile stub and DM folder
```gherkin
Given alice exists in workspace
When I run `paperwork invite bob`
Then `.paperwork/profiles/bob.md` exists (stub)
And `.paperwork/dm/alice--bob/` directory exists
And `.paperwork/dm/alice--bob/meta.md` lists alice and bob as participants
And `.paperwork/dm/alice--bob/thread.md` exists (empty)
And contacts.md includes both alice and bob
```

### Scenario: DM pair naming is deterministic
```gherkin
Given agents "zara" and "alice"
When a DM is created between them
Then the folder is named "alice--zara" (alphabetical sort)
```

### Scenario: List contacts
```gherkin
Given agents alice and bob exist in workspace
When I run `paperwork contacts`
Then output shows a table with alice and bob
And each row shows agent name and profile path
```

---

## Feature: DM Communication

### Scenario: Send a message
```gherkin
Given alice and bob have a DM thread
When alice runs `paperwork dm bob send "Hello, ready to start?"`
Then thread.md contains a message with seq 1, sender alice, body "Hello, ready to start?"
And the message has a valid ISO-8601 timestamp
```

### Scenario: Read messages by range
```gherkin
Given a DM thread with 5 messages
When I run `paperwork dm bob read --from 3 --to 4`
Then only messages with seq 3 and 4 are displayed
```

### Scenario: Reply with implicit mention
```gherkin
Given message #1 from alice exists
When bob runs `paperwork dm alice send "Yes!" --reply-to 1`
Then the new message has **Reply-To**: #1
And **To** includes alice
```

### Scenario: Mention triggers notification
```gherkin
Given alice and bob have a DM
When alice runs `paperwork dm bob send "Check this" --mention bob`
Then `.paperwork/notifications/bob/unread.md` contains an entry from alice
```

### Scenario: DM summary
```gherkin
Given alice and bob have a DM thread with 5 messages, last from bob
When I run `paperwork dm bob summary`
Then output shows: message count (5), last sender (bob), last timestamp
And snippet previews of recent messages are shown
```

### Scenario: Self-edit own message via CLI
```gherkin
Given alice sent message #3 with body "draft text"
When alice runs `paperwork dm bob edit 3 "final text"`
Then message #3 body is updated to "final text"
And seq, sender, timestamp remain unchanged
And no other messages are affected
```

---

## Feature: Post (GDM) Communication

### Scenario: Create a post
```gherkin
Given agents alice, bob, charlie exist
When I run `paperwork post create standup --participants alice,bob,charlie --title "Daily Standup"`
Then `.paperwork/posts/standup/meta.md` exists with all participants
And `.paperwork/posts/standup/log.md` exists (empty)
```

### Scenario: Multiple agents send to a post
```gherkin
Given post "standup" with participants alice, bob
When alice sends "Done with parser" to standup
And bob sends "Starting lexer" to standup
Then log.md contains seq 1 from alice and seq 2 from bob
```

### Scenario: Post summary
```gherkin
Given post "standup" with 10 messages, last from bob
When I run `paperwork post standup summary`
Then output shows: title, participant count, message count (10), last sender (bob), last timestamp
And snippet previews of recent messages are shown
```

### Scenario: Read post messages by range
```gherkin
Given post "standup" with 5 messages
When I run `paperwork post standup read --from 2 --to 4`
Then only messages with seq 2, 3, and 4 are displayed
```

### Scenario: List all posts
```gherkin
Given posts "standup" and "design-review" exist
When I run `paperwork post list`
Then output lists both posts with their titles
```

---

## Feature: Read Manifests

### Scenario: Create and populate a manifest
```gherkin
Given an initialized workspace
When I run `paperwork manifest create onboarding --description "How to read this project"`
And I run `paperwork manifest onboarding add --path "src/main.rs" --regex "fn main" --groups "0"`
Then `.paperwork/manifests/onboarding.md` exists with one entry
And the entry has a valid SHA-256 hash of src/main.rs
```

### Scenario: Verify fresh manifest
```gherkin
Given a manifest entry for src/main.rs with correct hash and matching regex
When I run `paperwork manifest onboarding verify`
Then the entry reports "Fresh"
```

### Scenario: Detect stale manifest
```gherkin
Given a manifest entry with regex "fn old_function" that no longer exists in the file
When I run `paperwork manifest onboarding verify`
Then the entry reports "Stale"
```

### Scenario: Detect shifted manifest
```gherkin
Given a manifest entry whose regex still matches but file hash has changed
When I run `paperwork manifest onboarding verify`
Then the entry reports "Shifted"
```

### Scenario: Read manifest selectively
```gherkin
Given a manifest with 5 entries
When I run `paperwork manifest onboarding read`
Then a table of contents (entry titles + paths) is displayed first
When I run `paperwork manifest onboarding read --full`
Then all entry details including notes are displayed
```

### Scenario: Remove a manifest entry
```gherkin
Given manifest "onboarding" with entry titled "Main entry point"
When I run `paperwork manifest onboarding remove "Main entry point"`
Then the entry is removed from the manifest file
And other entries remain unchanged
```

### Scenario: List all manifests
```gherkin
Given manifests "onboarding" and "api-map" exist
When I run `paperwork manifest list`
Then output lists both manifests with their descriptions
```

---

## Feature: Scope Query (Who)

### Scenario: Query ownership
```gherkin
Given alice owns "src/parser/**" and bob owns "src/lexer/**"
When I run `paperwork who --owns "src/parser/mod.rs"`
Then output shows: alice (owns)
And bob is not listed
```

### Scenario: Overlapping ownership surfaces all
```gherkin
Given alice owns "src/**" and bob owns "src/parser/**"
When I run `paperwork who --owns "src/parser/lib.rs"`
Then output shows both alice and bob with (owns) annotation
```

### Scenario: Query read access
```gherkin
Given alice has read scope "docs/**"
When I run `paperwork who --reads "docs/guide.md"`
Then output shows: alice (reads)
```

### Scenario: Query write access
```gherkin
Given bob has write scope "src/lexer/**"
When I run `paperwork who --writes "src/lexer/token.rs"`
Then output shows: bob (writes)
```

---

## Feature: Notifications

### Scenario: View unread notifications
```gherkin
Given bob has 2 unread notifications
When I run `paperwork notify --agent bob`
Then both notifications are displayed with sender, thread, seq, and snippet
```

### Scenario: Acknowledge notifications
```gherkin
Given bob has 2 unread notifications
When I run `paperwork notify --agent bob --ack`
Then `.paperwork/notifications/bob/unread.md` is empty
And `.paperwork/notifications/bob/history.md` contains the 2 notifications
```

---

## Feature: Append-Only Semantics

### Scenario: No message deletion
```gherkin
Given a thread with messages 1-5
Then there is no CLI command to delete a message
And the thread file can only grow (append) or have last-own-message edited
```

### Scenario: Self-edit own message
```gherkin
Given alice sent message #3
When alice runs `paperwork dm bob edit 3 "updated body"`
Then the body of #3 is updated in-place
And seq, sender, timestamp remain unchanged
And no other messages are affected
```
