# Evolve AI Orchestrator

> **Orchestrateur Rust pour IA évolutionnaire multi-agents**

## 🚀 Introduction

Ce projet est une **plateforme d’évolution artificielle** à grande échelle, entièrement écrite en Rust, conçue pour faire évoluer, surveiller et orchestrer des milliers d’agents IA autonomes (type “génome+code”), capables de muter, coopérer, survivre ou mourir selon une logique darwinienne.
L’architecture sépare **strictement** la logique “génétique” (mutations internes de chaque agent) et l’environnement externe (orchestrateur, règles, monitoring).

---

## 🧬 Principes clés

* **Chaque agent possède son propre génome (`genome.bin`)** décrivant quels modules/fichiers/fonctions sont “actifs” dans le code.
* **L’agent mute son génome de façon autonome** (mutation, activation/désactivation de parties du code, recombinaison, etc.)
* **L’orchestrateur NE TOUCHE PAS à la logique métier** : il synchronise la structure du code (mod.rs, main.rs, imports) pour qu’elle reflète l’état du génome.
* **L’orchestrateur monitore, valide, compile, exécute, tue ou relance les agents** selon des critères évolutifs, de sécurité, et de viabilité.
* **Tout est massivement parallélisé** (création, scan, gestion, monitoring) pour supporter des milliers d’agents.

---

## 📁 Structure du projet

```
Evolve_AI/
├── orchestrator/           # Binaire orchestrateur principal (pipeline de gestion/monitoring)
│   ├── src/
│   └── Cargo.toml
├── agents/                 # Tous les agents évolutifs générés
│   └── <agent_uuid>/       # 1 dossier par agent, cloné du template
│       ├── src/
│       ├── genome.bin      # Génome de l’agent (état, mutations, etc.)
│       └── Cargo.toml
├── agent_template/         # Template de base cloné pour chaque nouvel agent
│   └── src/
├── pipelines/              # Tous les scripts/process “pipelines” principaux (init, life cycle…)
│   └── ...
├── logs/                   # Logs, audits, rapports
│   └── initialization_log.txt
├── workspace/              # (optionnel) Config, métadonnées globales, etc.
│   └── ...
```

---

## 🏗️ Fonctionnement général

### 1. **Initialisation (`initiate_project`)**

* Création du dossier `agents/`
* Scan du template pour détecter les modules/fichiers/fonctions possibles
* Génération parallèle de N agents avec :

  * UUID unique
  * Dossier cloné à partir de `agent_template/`
  * Génome initial (`genome.bin`) généré random, avec des modules/fichiers/fonctions activés/inactivés
  * Mise à jour de `Cargo.toml` (identifiant unique)
  * Synchronisation automatique du code source (mod.rs, main.rs) pour refléter l’état du génome
  * Génération d’un hash/empreinte de code post-synchronisation
  * Log détaillé dans `logs/initialization_log.txt`

### 2. **Cycle de vie (pipeline LifeManager)**

* Boucle de gestion principale (massivement parallèle, chunkée pour la scalabilité)
* Pour chaque agent actif :

  * **Scan du code** pour détecter toute mutation ou modification du génome
  * **Vérification de la sécurité** (sandbox, signatures, etc.)
  * **Calcul de métriques structurelles** (complexité, mutations, nombre de fonctions, etc.)
  * **Validation et compilation** du code (désactive l’agent si build KO)
  * **Restart dynamique** si le code ou le génome a changé (kill/reload du process)
  * **Sélection naturelle** (critères Darwin personnalisables, mort/survie)
  * **Synchronisation wiring** (mod.rs, main.rs) avec le génome courant
  * **Notifications et hooks d’event** (création, mort, désactivation, etc.)
  * **Monitoring process** (status, crash, backoff, logs, ressources…)
  * **Sauvegarde de l’état** (listing global agents, logs, hashes, etc.)
  * **Sleep courte et relance**

---

## 🧬 Schéma général du cycle de vie (ASCII)

```
+---------------------------------------------------------+
|                   ORCHESTRATEUR PRINCIPAL              |
+---------------------------------------------------------+
        |                                              ^
        | [scan génome + code]                         |
        v                                              |
+---------------+        [mutation auto]         +-----------------+
|   AGENT 1     |<------------------------------|   AGENT N       |
|  genome.bin   |    ...     (coop possible)    |  genome.bin     |
|  src/         |------------------------------>|  src/           |
+---------------+                               +-----------------+
        |                                              ^
        | [synchronisation structurelle :              |
        |  mod.rs / main.rs / imports]                 |
        v                                              |
  [Compilation / Execution] <--------------------------+
        |
        v
  [Monitoring, Selection, Crash Handling, Metrics, ...]
        |
        v
  [Relance, désactivation, logs, etc.]
```

---

## 🗂️ Détail des fichiers et dossiers

### **orchestrator/**

* Pipeline principal de gestion du cycle de vie
* Boucle infinie, agents chunkés et traités en parallèle
* Chargement du listing global des agents
* Centralisation des logs, hooks d’event

### **agent\_template/**

* Modèle de base cloné pour chaque agent
* Doit contenir les modules et fonctions “possibles” (la diversité initiale de l’écosystème)
* Peut être enrichi à la main ou automatiquement (pool d’innovation)

### **agents/<uuid>/**

* Instance unique d’agent IA
* `src/` : code Rust complet de l’agent, généré/branché selon son génome
* `genome.bin` : sérialisation bincode du génome courant (statut de chaque fichier/fonction)
* `Cargo.toml` : manifest Rust unique
* Logs individuels (optionnel)

### **logs/**

* Logs d’initialisation, d’exécution, de crashs, de monitoring
* Pour debug, analytics, visualisation, audit

---

## 📦 Format du génome (exemple simplifié)

```ron
// genome.bin (bincode, structure logique)
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

**Le génome décrit : fichiers, modules, fonctions, statut d’activation.**
Toute mutation/récombinaison/modification ne touche QUE ce fichier, le reste du code est généré automatiquement.

---

## 🏁 Démarrage rapide

### **Prérequis**

* Rust (édition 2021)
* Rayon, Parking\_lot, Bincode, RON, etc. (voir Cargo.toml)

### **Initialiser le projet**

```sh
cargo run --bin orchestrator -- --init 1000
```

*Crée 1000 agents IA à partir du template, génomes randomisés, logs et listing global générés.*

### **Lancer le pipeline de gestion**

```sh
cargo run --bin orchestrator -- --life-cycle
```

*Démarre la boucle infinie de gestion du cycle de vie, relance, mutations, sélection, etc.*

---

## 🔁 Extension et scaling

* **Ajouter un nouveau module/fichier/fonction** au pool d’évolution : ajouter au template, il sera automatiquement disponible lors de la génération ou mutation d’un agent.
* **Customiser les hooks d’event** (ex : analytics, visualisation, contrôle, etc.) via `crate::notifications::notifier`
* **Modifier la sélection naturelle** (fitness, coop, adversité, énergie…) via `natural_selection::process_natural_selection`
* **Brancher un dashboard ou une API** : lire les logs, le listing, ou ajouter des hooks sur les events clés (création, mort, mutation, etc.)
* **Scaling horizontal** : chunking/parallelisation via Rayon permet de dépasser plusieurs milliers d’agents, scaling en cluster possible.

---

## 🧩 Points d’entrée principaux (code)

* **`initiate_project()`** : Génère, initialise, et configure les agents
* **`LifeManager::manage_agents_lifecycle()`** : Boucle Darwin de gestion de la population
* **`genome_sync::*`** : Sync entre le génome et le code source
* **`agent_scan_update::*`** : Scan et analyse de chaque agent
* **`natural_selection::*`** : Critères darwiniens/fitness (personnalisables)

---

## 🔒 Sécurité et monitoring

* **Agents isolés (processus, espace de fichiers)**
* **Vérification de la sécurité du code à chaque mutation (sandbox, signatures, audits)**
* **Logs centralisés (initialisation, crash, désactivation, etc.)**
* \*\*Backoff sur crash/désactivation (pas de redémarrage infini)
* **Hooks de notification/monitoring intégrables** (prometheus, webhooks, etc.)

---

## ⚡ FAQ / Questions fréquentes

**Q : Puis-je faire évoluer la structure des agents en live ?**

> Oui, chaque mutation de génome (par l’agent lui-même) sera prise en compte lors de la prochaine boucle, le wiring sera mis à jour automatiquement.

**Q : Peut-on créer des agents totalement nouveaux à la volée ?**

> Oui, en relançant l’initialisation avec un nouveau nombre d’agents, ou via un pipeline externe.

**Q : L’orchestrateur peut-il modifier le code métier ?**

> Non, seul l’agent gère la mutation de son génome. L’orchestrateur ne touche qu’à la structure (mod.rs, main.rs, imports) pour coller à l’état du génome.

**Q : Peut-on plugger d’autres stratégies d’évolution ?**

> Oui, tout est factorisé, il suffit d’implémenter de nouveaux modules/fonctions et de modifier le pipeline Darwin (fitness, coopération, etc.).

**Q : Comment monitorer l’ensemble ?**

> Les logs, l’état global, et les events sont centralisés (logs/ et listing). Pour un dashboard, pluggez Prometheus, Grafana, ou une simple API web.

---

## 🏆 Recommandations pro / “Best practices”

* **Dockerisez** l’ensemble pour la prod/multi-machine
* **Externalisez** les logs sur un cluster ou un service S3 pour l’analytics
* **Branchez un outil de monitoring/alerting** (Prometheus, Grafana, webhooks)
* **Benchmarquez le scaling** (nombre d’agents, RAM, CPU, taux de mutation, crash, survie)
* **Gardez toujours un backup du listing et des génomes**
* **Documentez vos hooks d’event pour faciliter l’extension**

---

## 📖 Pour aller plus loin

* Ajouter des modules d’auto-apprentissage, d’évolution de structure (mémoire graphique, reasoning, etc.)
* Simuler des environnements différents (coopération, compétition, ressources…)
* Générer dynamiquement de nouveaux modules ou fonctions via évolution “ouverte”
* Visualiser la population (dashboards, graphes, réseaux…)

---

## ✍️ Schéma d’architecture (ASCII)

```
        +---------------------+
        |    Orchestrator     |
        |  (Pipeline Life)    |
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

## 👷‍♂️ Crédits / Auteurs

Projet initié et conçu par Rémi Bezot
Contributions, feedbacks, issues et forks bienvenus !

---
