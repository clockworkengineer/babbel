# Concrete Plan: Documentation Overhaul, Harmonization, and Lifecycle Management

## Executive Summary

Following extensive architectural consolidation and code refactoring across the Babbel workspace (including the unification of domain format crates under `babbel_*`, implementation of embedded systems zero-allocation primitives, streaming pull parsers, format conversion pipelines, and the input sources overhaul), the repository's documentation has accumulated **broken links**, **phantom references**, **outdated dependency claims**, **stale repository URLs**, and **missing user-facing guides**.

This plan provides a concrete, step-by-step roadmap to:
1. **Add new, essential documentation** to fill critical information gaps (`docs/README.md`, `docs/EMBEDDED_GUIDE.md`, `docs/SECURITY.md`, `docs/DEVELOPMENT_GUIDE.md`, `docs/CODE_OF_CONDUCT.md`).
2. **Modify existing documentation** to accurately reflect current source capabilities, remove deprecated references, and repair all broken markdown links across the workspace.
3. **Audit and remove irrelevant, redundant, or orphaned documents** to maintain a clean, professional, and authoritative documentation surface.

---

## 1. Codebase Reality vs. Documentation Gap Analysis

### 1.1 Summary of Document Inventory

| Location / File | Current Status | Deficiencies Identified | Action Required |
| :--- | :--- | :--- | :---: |
| `README.md` (root) | Stale links | Links to deleted `docs/EMBEDDED_SYSTEMS_PLAN.md`; lacks references to new documentation. | **Modify** |
| `crates/json/README.md` | Broken links & stale roadmap | Links to non-existent `../docs/README.md`, references non-existent Code of Conduct & Embedding Guide; roadmap lists features already implemented (streaming, canonicalization). | **Modify** |
| `crates/yaml/README.md` | Outdated metadata | Claims dependency on `rand` (removed); contains dead repository URLs (`github.com/clockworkengineer/yaml.git`); lacks new uniform APIs. | **Modify** |
| `crates/bencode/README.md` | Stale URLs | Contains dead repository URL (`github.com/clockworkengineer/bencode`); omits 64 MB security limit documentation. | **Modify** |
| `crates/xml/README.md` | Outdated edition | Badge claims `edition-2021` (upgraded to `2024`); omits recent `IStatefulStream` integration. | **Modify** |
| `crates/babbel/README.md` | Deprecated naming | Feature table describes `json_lib`, `yaml_lib`, `xml_lib`, `bencode_lib` instead of `babbel_*`; omits `ConversionOptions`. | **Modify** |
| `crates/babbel_core/README.md` | Incomplete features | Missing docs for `ReaderSource`, `open_with_limit`, `SaveState`, `IStatefulStream`. | **Modify** |
| `docs/CONTRIBUTING.md` | Obsolete instructions | Mandates `--jobs 2` on Windows due to file collisions (fixed by `temp_dir()` test isolation). | **Modify** |
| `docs/ARCHITECTURE.md` | Mostly current | Missing references to `ReaderSource` and stateful stream snapshots. | **Modify** |
| `docs/BENCHMARKS_AND_MEMORY.md` | Current | Accurate node sizes; needs links to embedded guide. | **Modify** |
| `docs/CONVERSION_MATRIX.md` | Current | Accurate $O(N)$ pipeline; needs `ConversionOptions` examples. | **Modify** |
| `docs/TEXT_SUPPORT_GUIDE.md` | Current | Well structured; needs bidirectional navigation links. | **Modify** |
| `docs/README.md` | **Missing** | Central documentation hub linked by crate READMEs does not exist. | **Add** |
| `docs/EMBEDDED_GUIDE.md` | **Missing** | Code implements `StackBuffer`, `ArrayVecDestination`, `JsonPullParser`, etc., but user guide was deleted. | **Add** |
| `docs/SECURITY.md` | **Missing** | No centralized documentation of Billion Laughs mitigation, 64 MB DoS limits, or disclosure policies. | **Add** |
| `docs/DEVELOPMENT_GUIDE.md` | **Missing** | Referenced in `crates/json/README.md` but missing. | **Add** |
| `docs/CODE_OF_CONDUCT.md` | **Missing** | Referenced in `crates/json/README.md` but missing. | **Add** |
| `REFACTOR_PLAN.md` (root) | Redundant | Identical copy exists in `docs/REFACTOR_PLAN.md`. | **Consolidate** |
| `SOURCES_REFACTOR_PLAN.md` (root)| Redundant | Identical copy exists in `docs/SOURCES_REFACTOR_PLAN.md`. | **Consolidate** |
| `notes/*.md` | Gitignored internal notes | Internal planning artifacts; keep gitignored or archive. | **Preserve** |

---

## 2. Plan: New Documents to Add

### 2.1 `docs/README.md` (Documentation Hub & Sitemap)
- **Purpose**: Serve as the comprehensive index for all documentation in Babbel, resolving broken links from `crates/*/README.md`.
- **Contents**:
  - Sitemap linking to Architecture, Embedded Guide, Text Processing, Conversion Matrix, Benchmarks, Security, Contributing, and Development Guide.
  - Crate-level navigation with descriptions of all 6 crates.
  - Quick-reference table of common tasks (parsing, converting, embedded use, writing custom codecs).

### 2.2 `docs/EMBEDDED_GUIDE.md` (Embedded & Zero-Allocation Systems Guide)
- **Purpose**: Document the extensive embedded and `no_std` features implemented in `babbel_core::embedded` and across the format crates.
- **Contents**:
  - `no_std` configuration and `alloc` requirements.
  - Fixed-size memory allocations: `StackBuffer<const N>`, `MemoryTracker`, `FixedSizeBuffer`.
  - Zero-allocation destinations: `SliceDestination<'a>` and `ArrayVecDestination<const N>`.
  - Streaming Pull Parsers: `JsonPullParser`, `XmlPullParser`, `CsvPullParser`, and `IniPullParser` with code examples demonstrating document traversal under 16 KB RAM.
  - Memory bounds and error handling with `CompactError` (8 bytes).

### 2.3 `docs/SECURITY.md` (Security Policy & Defensive Design)
- **Purpose**: Detail security architecture, memory safety guarantees, input validation limits, and vulnerability reporting procedures.
- **Contents**:
  - Defensive limits:
    - Bencode length-prefix validation with 64 MB ceiling against memory exhaustion.
    - XML Billion Laughs and quadratic entity expansion mitigations (`entity_expansion_limit`, max depth).
    - File input guards (`FileSource::open_with_limit`, default 64 MB).
  - Path traversal and file access safety.
  - Responsible vulnerability disclosure policy and contact instructions.

### 2.4 `docs/DEVELOPMENT_GUIDE.md` (Developer & Contributor Guide)
- **Purpose**: Provide full developer onboarding covering workspace layout, tooling, profiling, and debugging.
- **Contents**:
  - Workspace structure and cross-crate dependency rules.
  - Toolchain requirements (Rust 1.88+, 2024 Edition).
  - Recommended developer commands (`cargo check`, `cargo test`, `cargo clippy`, `size_checks`).
  - AST node size verification and memory layout constraints.
  - Release build optimization and LTO profiling.

### 2.5 `docs/CODE_OF_CONDUCT.md` (Contributor Code of Conduct)
- **Purpose**: Standard Contributor Covenant (v2.1) establishing community standards.

---

## 3. Plan: Existing Documents to Modify

### 3.1 Root `README.md`
- Fix broken link: `docs/EMBEDDED_SYSTEMS_PLAN.md` $\rightarrow$ `docs/EMBEDDED_GUIDE.md`.
- Add links to `docs/README.md`, `docs/SECURITY.md`, and `docs/DEVELOPMENT_GUIDE.md`.
- Update feature highlights to emphasize new `no_std` pull parsers and uniform API verbs (`from_str`, `to_string`).
- Ensure all repository badges and links use the canonical `clockworkengineer/babbel` repository.

### 3.2 `crates/json/README.md`
- Fix broken documentation links (`../docs/README.md` $\rightarrow$ working docs hub).
- Document recently added features:
  - RFC 6901 JSON Pointer (`Node::pointer`, `Node::pointer_mut`).
  - RFC 7396 JSON Merge Patch (`Node::merge_patch`, `create_merge_patch`).
  - JSON Lines streaming (`lines::parse_json_lines`, `lines::to_json_lines`).
  - JSON5 comment stripping (`strip_comments`).
  - Uniform verbs: `babbel_json::from_str`, `from_bytes`, `to_string`, `to_vec`.
- Clean up "Roadmap ideas" section: remove features that have already been implemented.

### 3.3 `crates/yaml/README.md`
- Remove erroneous claim: *"Minimal dependencies - Only `rand` for testing utilities"*; update to declare **0 external dependencies** for production and tests.
- Fix broken repository clone URLs from `github.com/clockworkengineer/yaml.git` $\rightarrow$ `github.com/clockworkengineer/babbel.git`.
- Document new standardized API functions (`babbel_yaml::from_str`, `to_string`, `to_vec`, `to_destination`).
- Note integration with `babbel_core::io` traits (`IIndentationAware`, `IStatefulStream`, `SaveState`).

### 3.4 `crates/bencode/README.md`
- Fix repository badge and clone URLs to `clockworkengineer/babbel`.
- Document new standardized API functions (`babbel_bencode::from_str`, `to_string`, `to_vec`).
- Document the 64 MB maximum length prefix security limit.
- Document deduplicated memory utilities (`MemoryTracker`, `StackBuffer`) re-exported from `babbel_core`.

### 3.5 `crates/xml/README.md`
- Update Rust edition badge: `edition-2021` $\rightarrow$ `edition-2024`.
- Document new standardized API functions (`babbel_xml::from_str`, `to_string`, `to_vec`, `to_destination`).
- Document `IStatefulStream` implementation and pull parser integration.

### 3.6 `crates/babbel/README.md`
- Update feature flag descriptions: replace deprecated `*_lib` naming with `babbel_json`, `babbel_yaml`, `babbel_xml`, `babbel_bencode`.
- Document `ConversionOptions` (pretty printing, custom indentation) in `babbel::convert`.
- Document top-level ergonomic re-exports (`babbel::json`, `babbel::yaml`, etc.).

### 3.7 `crates/babbel_core/README.md`
- Add documentation for `ReaderSource<R: Read>` with internal buffering.
- Add documentation for `FileSource::open_with_limit` (64 MB DoS limit).
- Add documentation for `SaveState` and `IStatefulStream`.
- Document embedded primitives available under `babbel_core::embedded`.

### 3.8 `docs/CONTRIBUTING.md`
- Remove obsolete Windows `--jobs 2` warning in section 3 (tests now use isolated temp directories and run in full concurrency).
- Add link to `docs/CODE_OF_CONDUCT.md`.

---

## 4. Plan: Document Lifecycle, Cleanup & Deduplication

### 4.1 Root vs. `docs/` Consolidation
Currently, architectural plan documents exist simultaneously in both the repository root and in `docs/`:
- `REFACTOR_PLAN.md` (root) $\leftrightarrow$ `docs/REFACTOR_PLAN.md`
- `SOURCES_REFACTOR_PLAN.md` (root) $\leftrightarrow$ `docs/SOURCES_REFACTOR_PLAN.md`

**Decision**:
- Keep authoritative copies inside `docs/` (`docs/REFACTOR_PLAN.md`, `docs/SOURCES_REFACTOR_PLAN.md`).
- For root directory files, retain a clear reference pointer or consolidate to prevent drift, ensuring that root only houses high-level workspace files (`README.md`, `LICENSE`, `Cargo.toml`).
- Keep `notes/` ignored by git (as defined in `.gitignore`) for local developer scratch files.

---

## 5. Master Work Breakdown Structure & Target Files

| Phase | Target File | Action | Detailed Description | Status |
| :---: | :--- | :---: | :--- | :---: |
| **1** | `docs/README.md` | **[NEW]** | Central documentation hub, sitemap, and directory of guides. | 📋 Planned |
| **1** | `docs/EMBEDDED_GUIDE.md` | **[NEW]** | Complete guide for `no_std`, stack memory, and pull parsers. | 📋 Planned |
| **1** | `docs/SECURITY.md` | **[NEW]** | Threat model, DoS bounds (64MB, Billion Laughs), disclosure policy. | 📋 Planned |
| **1** | `docs/DEVELOPMENT_GUIDE.md` | **[NEW]** | Development workflow, toolchain, size assertions, release builds. | 📋 Planned |
| **1** | `docs/CODE_OF_CONDUCT.md` | **[NEW]** | Contributor Covenant v2.1. | 📋 Planned |
| **2** | `README.md` (root) | **[MODIFY]** | Fix broken links (`EMBEDDED_SYSTEMS_PLAN`), add sitemap links, update features. | 📋 Planned |
| **2** | `crates/json/README.md` | **[MODIFY]** | Fix `../docs/README.md` link, add pointers/patch/JSON5/lines, prune roadmap. | 📋 Planned |
| **2** | `crates/yaml/README.md` | **[MODIFY]** | Remove `rand` claim, fix repo URLs, document new APIs and core traits. | 📋 Planned |
| **2** | `crates/bencode/README.md` | **[MODIFY]** | Fix repo URLs, document 64MB limit and deduplicated memory tools. | 📋 Planned |
| **2** | `crates/xml/README.md` | **[MODIFY]** | Update edition badge to 2024, document stateful streams and pull parsing. | 📋 Planned |
| **2** | `crates/babbel/README.md` | **[MODIFY]** | Update feature table (remove `*_lib`), document `ConversionOptions`. | 📋 Planned |
| **2** | `crates/babbel_core/README.md` | **[MODIFY]** | Document `ReaderSource`, `open_with_limit`, `SaveState`, embedded module. | 📋 Planned |
| **2** | `docs/CONTRIBUTING.md` | **[MODIFY]** | Remove obsolete `--jobs 2` warning, link Code of Conduct. | 📋 Planned |
| **3** | Root plan cleanup | **[CONSOLIDATE]**| Ensure root plan files point cleanly to `docs/` to eliminate drift. | 📋 Planned |

---

## 6. Verification & Acceptance Criteria

1. **Link Verification**:
   - Every markdown link in `README.md`, `crates/*/README.md`, and `docs/*.md` resolves to a valid, existing file with zero 404s.
   - All relative links from crate subdirectories (`crates/*/README.md`) to `docs/` correctly use `../../docs/` or `../docs/`.
2. **Technical Accuracy**:
   - All code snippets in documentation reflect actual, compiling Rust code.
   - Dependencies stated in READMEs strictly match their respective `Cargo.toml`.
   - Feature flags documented in READMEs accurately reflect `[features]` in `Cargo.toml`.
3. **Repository Cleanliness**:
   - `git status` reflects a clean, well-organized documentation structure.
   - `cargo test --workspace --doc` passes 100% of executable doctests across all crates.
