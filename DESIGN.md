# fs-resources – Design

`fs-resources` knows what we have, who has time, what we need and who offers it. This repo holds the **program core**, no GUI. Contract:
`resources.capnp` in `fs-interfaces` v0.20.0 (draft, provisional: no standard for a resource model was found and no frontends are analysed). Program docs: `fs-documentation/docs/en/programs/apps/resources/`.

## State

Version 0.0.1 (+ round 3, branch `r3-types`): core, type description, a capnp server for the root, and a window (generated Admin frontend).

| Area | State |
|---|---|
| Resources, derived stock, reservations with a pure sweep, one write for `finish`, resumable delete | done |
| Capacity, allocations, shares: own unlogged store, acting person only, `as_actor` | done; the RPC server must refuse them until the transport knows the caller |
| `groupLoad` through `GroupLookup` | done, with the limits below |
| Needs and offers, reservation on accepting | done; see Open points |
| Type description (ADR 0066) of Resource, Reservation, Need, Offer, Capacity, Allocation; table in `describe/commands.rs` | done, see below |
| capnp server: `types`, `capabilities`, `supportReport` | done (crate `fs-resources-server`) |
| capnp server: `items`, `reservations`, `capacity`, `needs`, `subscribe` | **Unsupported**: not served (Cap'n Proto answers unimplemented); `capacity` must stay refused until the transport knows the caller |
| Window `fs-resources` (crate `fs-resources-app`, binary crate `fs-resources`) | done: shell (ADR 0065) + `fs-admin-frontend` over the catalog, core in process |
| costs, MSPDI, skills | not built |
| Tests | 115, real stores |

Checked by an independent reviewer over four rounds before the tag.

## Decisions (the owner may change them; all provisional)

### Answers to the page's open questions
1. **Things and people's time are one program,** because they share planning; the research note suggests the same. They stay in separate modules and separate store objects.
2. **ERP:** availability here, with a plain stock number for material. No warehouses, batches or valuation. **Costs** are out until they are analysed.

### Objects
Store types `Resource`, `Reservation`, `Need`, `Offer`, `Capacity`, `Allocation`, `GroupShare`; `context=resources`; IRIs `https://<instance>/resources/<id>`. No AS2 type exists for them, so content is in `fs:` members (namespace `https://freesynergy.net/ns#`). Index fields for every filter. Nothing here is an event or a task: those are only referred to by IRI (ADR 0050).

### Reservations
State pattern, one transition function. A pure sweep (Specification) computes what is held over time, and every check goes through it:
* `requested` holds nothing. `confirmed` holds `quantity - done` for `[from, to)`.
* `material` is used up: a confirmed reservation holds from `from` until finished, with no end, because what is used on Monday is gone on Tuesday.
* `confirm`, `change`, `setQuantity` and accepting an offer run under one lock across read and write, so two cannot overbook together. Clones share the lock.
* `finish` on a thing adds to `done` and frees that many; it ends when `done = quantity`; more than is out is refused. On material it lowers the stock by `quantity`, the rest returns, it ends at once. Refused once it has ended.
* **Stock is derived, so finishing is one write.** A material's stored value is its base; `quantity` = base minus the `done` of its ended reservations. `finish` writes only the reservation, so a crash cannot lower the stock twice or not at all. `setQuantity` compares `expected` with the derived value and stores the base that gives the new one.
* `free` returns stretches of equal free quantity over the asked range, adjacent equal stretches merged.
* A `room` is a thing with its own kind; the rules are those of a thing.
* Delete is refused while a confirmed reservation exists. Deleting first marks the resource: it freezes the stock and hides it from reads, then removes its reservations, then the resource. A retry resumes; a marker stays only until then.

### Capacity
The repo is built with the acting person's IRI (`actor`), and every `Capacities` method acts for that person.
`as_actor(iri)` gives the same repo for another person and shares its locks; a server uses it per caller, because separate repos on one store do not share locks.
* Another person's record answers exactly like an unknown id, including in `remove`, `answer` and `withdraw`.
* `propose` always returns `proposed`, whether the person exists or not.
* `answer` can be repeated. `withdraw` only by the proposer. Only `accepted` counts in `load`.
* `load` over a range: capacity is minutes per week, pro rata by overlap; allocations pro rata by overlap, integer arithmetic, rounding documented and tested.
* Capacity, allocations and shares are written with `UnloggedObjects`, **in a store of their own** that the host passes separately and never connects to the hub. The log of an ordinary write would carry them there. There is no undo for them, which is stated.
* The hub manifest of this program must not list these types in `writes`; provide a const list of writable types, with a test.
* **`groupLoad`:** the caller must be a member, otherwise it answers exactly like too few people. Only members who switched sharing on for this exact group and have a non-zero capacity in the week count. Port `GroupLookup` (`members(group)`, effective members); no port: `Unsupported`. Ranges are whole weeks on a fixed grid from a Monday 00:00 UTC; others are refused. Each week is summed and rounded on its own to a fixed 300 minutes, so a long range cannot be used to read a rate. A week with fewer than 5 contributors makes the whole answer `hidden`, with empty totals. In a week with fewer than 5 people with allocations, `allocationsHidden` is true and `allocatedMinutes` is 0. `people` is always 0, because a count would show who is absent. These numbers are constants here, not parameters.
  **Stated limits; the interface must say them, and a person agrees to them by sharing with that group:**
  * A person who stops sharing or is absent in a week changes the total by their hours, rounded, so comparing two weeks shows about what they contribute. Rounding hides small differences, not large ones: 3300 against 2400 for someone with 700.
  * A member who changes their own capacity in small steps can find a rounding edge and so learn the exact sum of the others in a week. Doing that in two weeks gives **the exact hours of someone absent in one of them** (run: 777). Rounding therefore does not protect against a member who sets out to do this; sharing with a group means trusting its members.
  * A one-minute change at a rounding edge moves the total by a whole step.
  * An attacker who fills a group with their own accounts, which can agree to share, gets close to one person's hours.
* The RPC server must refuse `Capacities` until the transport knows the caller (contract header). Told to thread 3.

### Needs and offers
State pattern for both. `covered` = sum of accepted offers, and the need becomes `covered` when that reaches `quantity`, and goes back to `open` when it falls below. Offers on a closed need are refused. Accepting an offer that names a resource, on a need with a time, makes a confirmed reservation in the same lock and stores its id; if that is not free, nothing changes. Withdrawing or declining an accepted offer cancels its reservation. Such a reservation cannot be cancelled directly, and a need's time cannot change while an offer is accepted. The offer is saved before its reservation changes; a repeated `decide` settles what a crash left unfinished. A need's holder for that reservation is its `subject`.

### Open points
* A fault while accepting an offer, followed by another action on that reservation and then a retried `decide`, can leave the offer accepted with its reservation cancelled, and the need covered. It needs a fault plus an intervention; the retry should detect it. Not fixed in 0.1.0.

### Feed
`subscribe` over the store's change feed: resources, reservations, needs, offers. Capacity types never arrive, because they are unlogged; tested.

### Not built
Costs, a time ledger, MSPDI import and export (fit unproven), skills, and scenarios. They have no contract method.

## Round 3: type description, server, window (ADR 0066)

### Pattern map (new parts)
| Where | Pattern | Why |
|---|---|---|
| `describe/commands.rs` `COMMANDS` | Registry as data | the single table of every command, with `supported`; description, backend refusal and the test against `requirements.md` read only it |
| `describe/spec.rs` | Value object + const builders | the tables read as lists |
| `describe/build.rs` | Adapter + Registry entry | table to `fs-types` description; `impl DescribesType` for the six stored types |
| `fs-resources-server` `root.rs` | Adapter | the root of `resources.capnp`; `types` serves `fs_types::CatalogServer` over `registry()` |
| `fs-resources-app` `backend/` | Repository (frontend trait) + Strategy per type | `CoreBackend` per data type, calling the core only through `Items`, `Reservations`, `Capacities`, `Needs` (`Ports`) |
| `fs-resources-app` `window.rs`, `gui.rs` | Facade | wiring only: no render code, no look (ADR 0066) |

### Types and standards
Six types are described: `Resource`, `Reservation`, `Need`, `Offer`, `Capacity`, `Allocation` (`GroupShare` is internal, no command). No open standard exists for a resource model, so each names "own model: resources.capnp". Fields follow `resources.capnp`.
**Unsupported (no basic type yet):** numbers (quantity, minutes) and booleans are `text` / `choice`; the core parses them and refuses what is not a number. A reference names one type only: `owner`, `holder`, `by`, `person` say `Person` though a group IRI is also valid; `subject` says `Task` though an event or project is also valid; `place` says `Place`.

### Commands
Every command of the contract and of `requirements.md` is in `COMMANDS` (42 rows). **Lists are `find`** (empty term = all, pages are walked by the backend; the core has no "all reservations" or "all offers" read, so those walk resources or needs). **`change` is an `Edit`** (field path, basic type, values; not in the edit = unchanged, no values = cleared); the backend refuses an edit of a field the core cannot change (`kind`, `state`, `resource`, `holder` ...) with a finding on it. Commands on a record: `TypeDescription::record_commands`.
The `supported` flag goes on the wire in `describe/build.rs` (`CommandDescription::unsupported()`), from the table. Switched from the local rule to the schema flag when `fs-interfaces` v0.0.11 appeared; `unsupported_commands()` and the backend refusal stay (the frontend asks `Backend::supported` too).

`requirements.md` has five rows, all `proposed`:
| Requirement | Commands | State |
|---|---|---|
| Generate list and form views from a type description | the generated frontend itself | done |
| Link records of one area to another by interface | `Resource.link` | **Unsupported** (supported = false) |
| Stock with minimum amount and due date, missing items to a list | `Resource.setMinimum`, `Resource.missing` | **Unsupported** |
| Check-out/check-in with status labels deciding assignability | `Resource.checkOut`, `checkIn`, `setStatus` | **Unsupported** (`finish` covers return, but no status labels exist) |
| Item classes: tagged, interchangeable, part of, used up | `Resource.setClass` | **Unsupported** (`kind` = thing/room/material only) |
`tests/describe.rs` reads a verbatim copy of the requirements page (`tests/fixtures/requirements.md`, copied 2026-11-10 from fs-documentation) and fails if a row is missing from the table or a command stands for no row; refresh the fixture when the page changes.

### The window
`fs-resources` without arguments opens the window: a screen per type, standard menus (ADR 0065). The core runs in this process (two stores: `objects.db` and the private `private.db` in `$XDG_DATA_HOME/freesynergy/resources`; `--memory` keeps nothing), the actor is the local owner (provisional, no sign-in yet). `fs-resources serve` runs the server.
* `fs-admin-frontend` is used at the commit named in `crates/fs-resources-app/Cargo.toml` (provisional, until a tag).
* A refusal by a rule of the core (overbooked, wrong state, overlap, no group lookup) is `Rejected` with no finding, because `RecordProblem` has no "the program's rule refused" code; the reason goes to stderr. Needs a code from thread T/A.
* A command that is not supported is refused by the backend with a finding on the command's name (`RecordProblem::UnknownField`): provisional stand-in for a typed Unsupported problem.
* Groups are named in the backend by `urn:uuid:<id>`: this program cannot know another program's IRI form.
* `capacity` acts for the actor the repo was built with; that is right for the in-process window and wrong for any server, which stays Unsupported until the transport knows the caller.
* Colour and style: the frontend's; none here. The existing demo in `fs-resources-core/examples` is text, not a render.

### Open
* Server methods for items, reservations, needs: not built (above).
* The offline refusal of `groupLoad` (no Groups program connected in the window) shows as "refused".
