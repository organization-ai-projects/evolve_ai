# Evolve AI Orchestrator

> **A Rust Evolutionary AI Multi-Agent Orchestrator**

## 🚀 Introduction

This project is a **large-scale artificial evolution platform**, fully written in Rust, designed to evolve, monitor, and orchestrate thousands of autonomous AI agents ("genome+code" style). These agents can mutate, cooperate, compete, survive, or die, following a Darwinian logic.
The architecture **strictly separates** the "genetic" logic (internal agent genome mutations) from the external environment (orchestrator, rules, monitoring).

---

## 🧬 Key Principles

* **Each agent has its own genome (`genome.bin`)** describing which modules/files/functions are "active" in the code.
* **Each agent mutates its genome autonomously** (mutation, activation/deactivation of parts of code, recombination, etc.)
* **The orchestrator NEVER touches business logic**: it only synchronizes code structure (mod.rs, main.rs, imports) to reflect the genome.
* **The orchestrator monitors, validates, compiles, runs, kills, or restarts agents** based on evolutionary, safety, and viability criteria.
* **Everything is massively parallelized** (creation, scanning, management, monitoring) to support thousands of agents.

---

## 📁 Project Structure

```
evolve_ai/
├── orchestrator/           # Main orchestrator binary (management/monitoring pipeline)
│   ├── src/
│   └── Cargo.toml
├── agents/                 # All generated evolutionary agents
│   └── <agent_uuid>/       # One folder per agent, cloned from the template
│       ├── src/
│       ├── genome.bin      # Agent genome (state, mutations, etc.)
│       └── Cargo.toml
├── agent_template/         # Base template cloned for each new agent
│   └── src/
├── pipelines/              # Main scripts/pipelines (init, life cycle, etc.)
│   └── ...
├── logs/                   # Logs, audits, reports
│   └── initialization_log.txt
├── workspace/              # (optional) Config, global metadata, etc.
│   └── ...
```

---

## 🏗️ General Workflow

### 1. **Initialization (`initiate_project`)**

* Creates the `agents/` folder
* Scans the template to detect possible modules/files/functions
* Generates N agents in parallel with:

  * Unique UUID
  * Folder cloned from `agent_template/`
  * Initial genome (`genome.bin`) randomly generated, with active/inactive modules/files/functions
  * Cargo.toml updated (unique identifier)
  * Automatic code sync (mod.rs, main.rs) to reflect genome state
  * Hash/fingerprint of code after sync
  * Detailed log in `logs/initialization_log.txt`

### 2. **Life Cycle (LifeManager pipeline)**

* Main management loop (massively parallel, chunked for scalability)
* For each active agent:

  * **Code scan** to detect genome/code mutations
  * **Security checks** (sandbox, signatures, etc.)
  * **Structural metrics calculation** (complexity, mutations, function count, etc.)
  * **Validation and compilation** (agent disabled if build fails)
  * **Dynamic restart** if code/genome changed (kill/reload process)
  * **Natural selection** (customizable Darwinian criteria, death/survival)
  * **Wiring sync** (mod.rs, main.rs) with current genome
  * **Notifications/event hooks** (creation, death, disable, etc.)
  * **Process monitoring** (status, crash, backoff, logs, resources...)
  * **State save** (global agent listing, logs, hashes, etc.)
  * **Short sleep and restart**

---

## 🧬 General Life Cycle Diagram (ASCII)

```
+---------------------------------------------------------+
|                   MAIN ORCHESTRATOR                    |
+---------------------------------------------------------+
        |                                              ^
        | [scan genome + code]                         |
        v                                              |
+---------------+        [auto-mutation]         +-----------------+
|   AGENT 1     |<------------------------------|   AGENT N       |
|  genome.bin   |    ...     (coop possible)    |  genome.bin     |
|  src/         |------------------------------>|  src/           |
+---------------+                               +-----------------+
        |                                              ^
        | [structural sync:                            |
        |  mod.rs / main.rs / imports]                 |
        v                                              |
  [Compilation / Execution] <--------------------------+
        |
        v
  [Monitoring, Selection, Crash Handling, Metrics, ...]
        |
        v
  [Restart, disable, logs, etc.]
```

---

## 🗂️ File and Folder Details

### **orchestrator/**

* Main management pipeline
* Infinite loop, chunked/parallel agent processing
* Loads global agent listing
* Centralized logs, event hooks

### **agent\_template/**

* Base model cloned for each agent
* Should contain all “possible” modules/functions (initial ecosystem diversity)
* Can be enriched manually or automatically (innovation pool)

### **agents/<uuid>/**

* Unique agent instance
* `src/`: full Rust code for the agent, generated/wired as per genome
* `genome.bin`: current genome serialization (bincode, state for each file/function)
* `Cargo.toml`: unique Rust manifest
* Individual logs (optional)

### **logs/**

* Initialization, execution, crash, and monitoring logs
* For debug, analytics, visualization, audit

---

## 📦 Genome Format (simplified example)

```ron
// genome.bin (bincode, logical structure)
GenomeConfig {
    files: [
        FileGene {
            path: "src/neural.rs",
            active: true,
            functions: { "run": true, "reset": false }
        },
        FileGene {
            path: "src/curiosity.rs",
            active: false,
            functions: { "explore": false }
        },
        // ...
    ]
}
```

**The genome describes: files, modules, functions, activation status.**
Any mutation/recombination/modification only changes this file; the rest of the code is generated automatically.

---

## 🏁 Quickstart

### **Requirements**

* Rust (edition 2021)
* Rayon, Parking\_lot, Bincode, RON, etc. (see Cargo.toml)

### **Initialize the project**

```sh
cargo run --bin orchestrator -- --init 1000
```

*Creates 1000 AI agents from the template, random genomes, logs, and global listing.*

### **Launch the management pipeline**

```sh
cargo run --bin orchestrator -- --life-cycle
```

*Starts the infinite management loop: relaunch, mutation, selection, etc.*

---

## 🔁 Extension and scaling

* **Add new modules/files/functions** to the evolutionary pool: just add to the template—they'll be available for new agents or mutations.
* **Customize event hooks** (analytics, visualization, control, etc.) via `crate::notifications::notifier`
* **Modify natural selection** (fitness, cooperation, adversity, energy, etc.) in `natural_selection::process_natural_selection`
* **Connect a dashboard or API**: read logs, listing, or add hooks for key events (creation, death, mutation, etc.)
* **Horizontal scaling**: chunking/parallelization via Rayon lets you run thousands of agents—scaling to clusters is possible.

---

## 🧩 Main Entry Points (code)

* **`initiate_project()`**: Generates, initializes, and configures agents
* **`LifeManager::manage_agents_lifecycle()`**: Darwinian population management loop
* **`genome_sync::*`**: Genome ↔ source code sync
* **`agent_scan_update::*`**: Per-agent scan and analysis
* **`natural_selection::*`**: Darwinian/fitness criteria (customizable)

---

## 🔒 Security & Monitoring

* **Agents isolated (process, file space)**
* **Code safety checked at every mutation (sandbox, signatures, audits)**
* **Centralized logs (init, crash, disable, etc.)**
* \*\*Crash/desactivation backoff (no infinite restart)
* **Notification/monitoring hooks available** (prometheus, webhooks, etc.)

---

## ⚡ FAQ / Common Questions

**Q: Can agents evolve their structure live?**

> Yes, every genome mutation (by the agent itself) is picked up on the next orchestrator loop; the wiring updates automatically.

**Q: Can I create entirely new agents on the fly?**

> Yes, by re-running initialization with a new agent count, or via an external pipeline.

**Q: Can the orchestrator modify business logic?**

> No, only the agent manages genome mutation. The orchestrator only syncs structure (mod.rs, main.rs, imports) to match genome state.

**Q: Can I plug in other evolutionary strategies?**

> Yes, everything is modular—just implement new modules/functions or modify the Darwin pipeline (fitness, cooperation, etc.).

**Q: How do I monitor everything?**

> Logs, global state, and events are centralized (logs/ and listing). For a dashboard, plug in Prometheus, Grafana, or a web API.

---

## 🏆 Pro Recommendations / Best practices

* **Dockerize** everything for prod/multi-machine
* **Externalize logs** to a cluster or S3 for analytics
* **Hook up monitoring/alerting** (Prometheus, Grafana, webhooks)
* **Benchmark scaling** (agent count, RAM, CPU, mutation rate, crash/survival)
* **Always back up listings and genomes**
* **Document your event hooks for easy extension**

---

## 📖 Going further

* Add self-learning, structural evolution modules (graph memory, reasoning, etc.)
* Simulate different environments (cooperation, competition, resources…)
* Dynamically generate new modules/functions via “open evolution”
* Visualize the population (dashboards, graphs, networks…)

---

## ✍️ Architecture Diagram (ASCII)

```
        +---------------------+
        |    Orchestrator     |
        |  (Life Pipeline)    |
        +---------------------+
                  |
                  v
    +----------------------------+
    |  agents/                   |
    |   +----+     +----+        |
    |   |A001| ... |A999| ...    |
    |   +----+     +----+        |
    |   |genome|   |genome|      |
    |   |src/  |   |src/  |      |
    +----------------------------+
                  |
                  v
    +----------------------------+
    |  logs/, listing, metrics    |
    +----------------------------+
```

---

## 👷‍♂️ Credits / Authors

Project initiated and designed by Bezot Rémi
Contributions, feedback, issues and forks are welcome!

---
