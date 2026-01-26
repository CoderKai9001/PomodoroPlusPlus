# Pomodoro++ TUI

**System Specification for Autonomous Coding Agent**

## 1. Overview

Pomodoro++ is a **terminal-based productivity application** implemented as a **TUI (Text User Interface)**.
It helps users:

* Run Pomodoro timers.
* Assign tags (subjects) to each session.
* Persist all sessions in a local SQLite database.
* Visualize productivity using:

  * Weekly and monthly graphs.
  * Tag-based analytics.
  * A GitHub-style heatmap.

The application must be written in:

* **Language:** Rust
* **TUI Framework:** ratatui (formerly tui-rs)
* **Database:** SQLite (rusqlite or sqlx)

This is a **fully interactive TUI**, not a static CLI.

---

## 2. High-Level Architecture

Use a layered architecture:

```
┌────────────────────────┐
│        UI Layer        │ ratatui widgets, layout
├────────────────────────┤
│      App State         │ in-memory state machine
├────────────────────────┤
│   Business Logic       │ pomodoro rules, stats
├────────────────────────┤
│    Data Access         │ SQLite queries
├────────────────────────┤
│    Persistence         │ sqlite database file
└────────────────────────┘
```

The app runs as a continuous event loop:

1. Capture keyboard input.
2. Update application state.
3. Execute timer/stat logic.
4. Render UI.

---

## 3. Core Features

### 3.1 Home Screen (Main Dashboard)

This is the default screen when the app launches.

#### Functional Requirements

The home screen must include:

1. **Pomodoro Timer**

   * Display remaining time.
   * Supports:

     * Start
     * Pause
     * Stop / Reset
   * Modes:

     * Work
     * Break

2. **Tag System**

   * User can:

     * Select an existing tag.
     * Create a new tag.
   * Each pomodoro session is associated with exactly one tag.

3. **Configuration**

   * User can modify:

     * Default work duration (e.g., 25 min).
     * Default break duration (e.g., 5 min).
   * Values are persisted.

4. **Controls**

   * Keyboard driven:

     * Start / pause / stop timer.
     * Navigate tags.
     * Open settings.

#### Suggested UI Layout

```
┌────────────────────────────────────┐
│ Pomodoro++                         │
├───────────────┬────────────────────┤
│ Timer         │ Tag                │
│               │ [ ML ] [ Math ]    │
│   24:13       │ [ + New Tag ]      │
│               │                    │
│ [Start] [||]  │                    │
│ [Stop]        │                    │
├───────────────┴────────────────────┤
│ Work: 25 min   Break: 5 min        │
└────────────────────────────────────┘
```

---

## 4. Statistics Page

This is a separate screen accessible via a keybinding.

### Functional Requirements

The statistics page must support:

1. **Tag-based Graphs**

   * Weekly time spent per tag.
   * Monthly time spent per tag.

2. **Global Graphs**

   * Total time across all tags.
   * Weekly and monthly views.

3. **Interactive Filtering**

   * User selects:

     * A tag OR "All tags".
     * Time range: Weekly / Monthly.
   * Only one graph is shown at a time.
   * Graph updates dynamically.

4. **Graph Types**

   * Bar chart or line chart.
   * Rendered using terminal characters (no images).

#### Example UI

```
┌────────────────────────────────────┐
│ Statistics                         │
├────────────────────────────────────┤
│ View: [ Weekly ] [ Monthly ]       │
│ Tag:  [ ML ]                       │
├────────────────────────────────────┤
│                                    │
│   ▂▅▇█▆▃▁                          │
│   Hours per day                    │
│                                    │
└────────────────────────────────────┘
```

---

## 5. Heatmap View

A separate screen showing a **GitHub-style activity heatmap**.

### Functional Requirements

* Shows daily pomodoro activity.
* Each cell = one day.
* Color/intensity = total minutes spent that day.
* Range:

  * At least last 6 months.
* Legend:

  * Low → High activity.

#### Example

```
Mon ░░▓▓▓
Tue ░▓▓▓▓
Wed ░░░▓▓
Thu ░▓▓▓░
Fri ░░▓▓░
```

---

## 6. Data Model

### SQLite Schema

#### sessions table

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT,
    end_time TEXT,
    duration INTEGER,
    tag TEXT,
    type TEXT
);
```

#### tags table

```sql
CREATE TABLE tags (
    name TEXT PRIMARY KEY
);
```

#### config table

```sql
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

---

## 7. App State Model

The application must maintain an in-memory state struct:

```rust
struct AppState {
    current_screen: Screen,
    timer_running: bool,
    mode: PomodoroMode,
    remaining_seconds: u64,
    selected_tag: String,
    work_duration: u64,
    break_duration: u64,
}
```

---

## 8. Business Logic

### Timer Logic

* When timer reaches zero:

  * Persist session.
  * Switch mode (work ↔ break).
  * Reset timer.

### Session Persistence

Each completed work session:

* Insert row into `sessions`.

---

## 9. Statistics Engine

The system must compute:

### Weekly per tag

```sql
SELECT DATE(start_time), SUM(duration)
FROM sessions
WHERE tag = ?
AND start_time >= DATE('now','-7 days')
GROUP BY DATE(start_time);
```

### Monthly per tag

```sql
SELECT STRFTIME('%Y-%m', start_time), SUM(duration)
FROM sessions
WHERE tag = ?
GROUP BY STRFTIME('%Y-%m', start_time);
```

### Heatmap

```sql
SELECT DATE(start_time), SUM(duration)
FROM sessions
GROUP BY DATE(start_time);
```

---

## 10. UI Navigation

Required keybindings:

| Key     | Action        |
| ------- | ------------- |
| `q`     | Quit          |
| `h`     | Home          |
| `s`     | Statistics    |
| `m`     | Heatmap       |
| `space` | Start / Pause |
| `r`     | Reset         |
| `t`     | Change tag    |
| `+`     | New tag       |

---

## 11. Rendering Requirements

Use **ratatui widgets**:

* `Block`
* `Paragraph`
* `List`
* `Chart`
* `BarChart`
* Custom heatmap widget

Rendering must be:

* Non-blocking.
* Redrawn at least once per second.

---

## 12. Project Structure

Suggested Rust module layout:

```
src/
 ├─ main.rs
 ├─ app.rs          // AppState
 ├─ ui/
 │   ├─ home.rs
 │   ├─ stats.rs
 │   └─ heatmap.rs
 ├─ db.rs           // SQLite
 ├─ timer.rs
 └─ stats_engine.rs
```

---

## 13. Non-Goals

The following are explicitly out of scope:

* Cloud sync.
* Mobile or web UI.
* Image-based plotting.
* Authentication.

---

## 14. Quality Requirements

The system must:

* Compile with stable Rust.
* Use idiomatic Rust patterns.
* Be modular and testable.
* Persist data across sessions.
* Never block the UI thread.

---

## 15. Success Criteria

The project is considered complete when:

* A user can run:

  ```
  pomodoro++
  ```
* Fully control a pomodoro timer from terminal.
* Assign tags.
* View weekly/monthly graphs.
* View a GitHub-style heatmap.
* Quit and reopen without losing data.

---

## Final Instruction to Coding Agent

> You are to implement the above system as a **fully functional Rust TUI application** using **ratatui and SQLite**, strictly following this specification.
> Prioritize correctness, modular design, and interactive usability over visual perfection.

---