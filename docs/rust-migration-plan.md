# wowdecomp Rust Migration Learning Plan

We will progressively replace C++ systems with Rust while keeping the client playable. The first milestone is **Rust-owned cooldown tracking, built through CMake with `cxx`, and validated in a running client connected to an emulated server**.

Starting reference: client revision `20f9120e`. Record the exact server revision and toolchain during step 1.

**How we will work**

The human coder writes all implementation code, tests, build files, and bindings. The assistant acts as a technical guide and reviewer: it proactively explains the actual code as it is written, answers questions, reviews changes, and helps interpret failures. Creating this plan does not authorize the assistant to implement it.

Assume working knowledge of C++ and basic Rust. Keep explanations concise and focus on what a function does, its inputs and outputs, state changes, dependencies, and how it connects to the game. Explain migration choices, behavior that must be preserved, and ownership or lifetime constraints when they matter. Cover complex syntax and unfamiliar Rust concepts when relevant or requested; skip elementary syntax explanations and quizzes.

Use a concrete example when it clarifies nontrivial behavior. Reserve line-by-line walkthroughs for complex code or a specific request. Explain what each check establishes and connect failures to their causes before the coder applies a fix. Each explanation should lead to a concrete next task. Offer the important reasoning proactively, with room for questions and deeper discussion.

Each session follows this loop:

1. Briefly explain the relevant behavior, dependencies, and migration implications.
2. Give one concrete coding or investigation task and explain its purpose.
3. The coder makes a coherent change. Review its behavior and integration, going deeper where needed.
4. The coder runs the suggested checks; the assistant explains the results and helps diagnose failures. The assistant can run checks when asked.
5. Review what changed, why it works, and what remains unclear before choosing the next task.

Use pseudocode or small examples when they help explain a design; complete implementations remain the coder's work. The assistant waits for the coder before moving to another task or step. Learning checkpoints guide discussion and review, and the pace follows the coder's understanding.

At the end of a session, record the current step, decisions, concepts learned, checks run, open questions, and the next small task. Mark a step complete when its check passes and questions about the behavior and implementation are resolved.

**Initial platform targets**

| Platform | What must work |
|---|---|
| Windows x64 | Full client with Rust enabled, including the executable used for packaging |
| Linux x64 | Rust gameplay module and a small C++ program exercising its `cxx` bridge |
| Windows Win32 | Existing C++ reference build |

These are native builds on each platform. A full Linux client and Linux-to-Windows cross-compilation are later milestones. Keep platform APIs in the C++ adapters so the Rust gameplay module can run on both systems.

**The steps**

1. **Understand the existing system and capture a baseline.**

   Read `SpellHistory` and follow one path from a received cooldown event to a cooldown query and UI update. Choose the emulated server and prepare a character with known spells. Record client/server revisions, toolchains, character setup, and a repeatable reset procedure. Work on a dedicated migration branch, keeping the C++ reference and decompilation workflow usable.

   **Learn:** which behavior belongs to the client, which belongs to the server, and where cooldown state lives today.

   **Check:** the unchanged client logs in, casts spells, reports cooldowns, and completes the baseline scenario reproducibly. Keep game assets and account credentials outside version control.

2. **Make a Rust function callable through the normal CMake build.**

   Create the Cargo workspace, one Rust static library, and a minimal `cxx` bridge. Pin Rust and dependency versions. Try pinned Corrosion integration first; use explicit CMake rules invoking Cargo and the `cxx` generator if the build proof fails. Record the CMake requirement of the chosen tooling.

   Connect generation, compilation, and linking to both `Wow` and `WowClient`. Establish compatible MSVC runtime and Cargo profile settings for Debug, Release, and RelWithDebInfo.

   **Learn:** Cargo's role, CMake's role, generated bindings, linking, and the difference between build profiles and the C runtime.

   **Check:** ordinary CMake builds include Rust automatically. Editing a Rust function or bridge regenerates and rebuilds the affected outputs. Both executable targets link, the Rust-off configuration works, and Win32 still builds. A tiny function such as `HasAura` is sufficient for this check.

3. **Exercise the bridge on Linux.**

   Build a small C++ executable that calls the same Rust library through `cxx`. Give it explicit test inputs without depending on the Windows client. Keep the Rust logic and bridge inputs portable.

   **Learn:** the difference between portable gameplay code and platform-specific client integration.

   **Check:** the C++ executable compiles, links, and calls Rust successfully on Linux and Windows. Run Rust unit tests on both platforms.

4. **Define a complete cooldown interface.**

   Inventory every `SpellHistory` call and every direct access to its lists. Route direct mutations through explicit operations while retaining C++ behavior. Define the Rust state owner, creation and destruction points, incoming events, metadata inputs, and query results.

   C++ continues handling packet decoding, database access, Lua, and UI updates. Pass timestamps and the required spell, category, item, pet, and rune information explicitly. Every cooldown read and mutation must have a route through the new interface.

   **Learn:** ownership, borrowed values, opaque Rust objects, and how to choose a boundary without reproducing the C++ object model.

   **Check:** a written interface accounts for all existing consumers and lifecycle operations. Any C++ preparation changes pass the baseline before Rust takes over state.

5. **Implement cooldown behavior in Rust in small increments.**

   Work through adding and querying entries, clearing and delaying them, expiration, shared categories, global cooldowns, and item, pet, and rune interactions. Port one operation at a time and review it before continuing. Rust owns the entries and calculations; C++ supplies the agreed inputs.

   Add focused tests for operation sequences, held cooldowns, timestamp wraparound, and ordering when cooldowns end at the same time. Preserve observable behavior before changing data structures or simplifying rules.

   **Learn:** Rust structs, collections, mutation, integer behavior, and tests through actual gameplay rules.

   **Check:** the completed service handles every operation from step 4, with deterministic tests using an explicit clock. Construct and destroy Rust state through the agreed client lifecycle.

6. **Compare the Rust and C++ implementations.**

   Feed independent C++ and Rust states identical events, metadata, and timestamps. Compare cooldown results, output flags, and state transitions. During live comparison, use the C++ result and apply networking and UI effects once.

   Record enough input to reproduce each mismatch. Introduce a temporary deliberate error to verify that the comparison detects it, then remove it.

   **Learn:** differential testing, reproducible failures, and why matching the C++ reference establishes migration parity rather than original-client fidelity.

   **Check:** valid-input scenarios have zero unexplained mismatches. Test unsupported inputs separately against the agreed contract.

7. **Validate the Rust system in the running game.**

   Build a repeatable launcher and scenario runner for the chosen server and Windows client. Reset the character, log in, cast spells, inspect client cooldown queries and UI events, and exercise server-issued changes and clears. Include expiry, recasting, item and pet cases, rune interactions, and logout/login cleanup.

   Capture inputs for replay and define timing tolerances for live observations. Compare the baseline and Rust client under the same setup, including frame time and memory use against budgets agreed before promotion.

   **Learn:** how to distinguish client behavior, server behavior, and timing differences.

   **Check:** scenarios pass with Rust serving the live cooldown results. Client state and UI behavior agree with expectations as well as server observations; server acceptance of a cast alone is insufficient.

8. **Promote Rust and retire the production C++ state.**

   Make Rust the default on x64 after the comparison and gameplay checks pass. Remove the old cooldown state and implementation from that production path. Keep a tested C++ reference configuration for Win32, comparisons, and rollback.

   Record which behavior moved, which bindings remain, and how future fixes reach both Rust and the retained reference. Document a regular process for incorporating upstream C++ fixes before choosing the next system.

   **Learn:** safe rollout, rollback, and maintaining a migration alongside ongoing development.

   **Check:** the packaged client uses Rust cooldown tracking, rollback works, Linux bridge tests pass, and the coder can explain the ownership and event flow.

**Files to start reading**

- [SpellHistory and cooldown data](../WoW/Source/Magic/MagicClient/Spell_C.h)
- [Cooldown implementation and packet handlers](../WoW/Source/Magic/MagicClient/Spell_C.cpp)
- [Lua cooldown queries](../WoW/Source/Ui/SpellBookFrame_Scripts.cpp)
- [Root CMake build](../CMakeLists.txt)
- [Packaged executable build](../BuildWaystone.bat)

**Tooling references**

- [CXX and CMake](https://cxx.rs/build/cmake.html)
- [Corrosion binding integration](https://corrosion-rs.github.io/corrosion/ffi_bindings.html)
- [Corrosion Windows runtime considerations](https://corrosion-rs.github.io/corrosion/common_issues.html)

**First session:** walk through the existing cooldown event and query path together, identify the server setup, and capture the unchanged client's behavior. Implementation begins with the coder's first agreed task.
